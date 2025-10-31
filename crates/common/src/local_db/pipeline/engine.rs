use crate::local_db::address_collectors::{collect_store_addresses, collect_token_addresses};
use crate::local_db::decode::{
    sort_decoded_events_by_block_and_log, DecodedEvent, DecodedEventData,
};
use crate::local_db::pipeline::{
    ApplyPipeline, BootstrapConfig, BootstrapPipeline, EventsPipeline, StatusBus, SyncConfig,
    SyncOutcome, TargetKey, TokensPipeline, WindowPipeline,
};
use crate::local_db::query::fetch_store_addresses::{fetch_store_addresses_stmt, StoreAddressRow};
use crate::local_db::query::{LocalDbQueryExecutor, SqlStatement};
use crate::local_db::LocalDbError;
use crate::rpc_client::LogEntryResponse;
use alloy::primitives::Address;
use std::collections::{BTreeSet, HashSet};
use url::Url;

/// Generic engine that orchestrates a full sync cycle by delegating
/// environment-specific behavior to the supplied pipeline adapters.
pub struct SyncEngine<B, W, E, T, A, S> {
    bootstrap: B,
    window: W,
    events: E,
    tokens: T,
    apply: A,
    status: S,
}

/// Runner-supplied inputs that are static for the duration of a sync cycle.
#[derive(Debug, Clone)]
pub struct SyncInputs {
    pub target: TargetKey,
    pub metadata_rpcs: Vec<Url>,
    pub cfg: SyncConfig,
    pub dump_str: Option<String>,
}

impl<B, W, E, T, A, S> SyncEngine<B, W, E, T, A, S>
where
    B: BootstrapPipeline,
    W: WindowPipeline,
    E: EventsPipeline,
    T: TokensPipeline,
    A: ApplyPipeline,
    S: StatusBus,
{
    pub fn new(bootstrap: B, window: W, events: E, tokens: T, apply: A, status: S) -> Self {
        Self {
            bootstrap,
            window,
            events,
            tokens,
            apply,
            status,
        }
    }

    /// Executes a single sync cycle using the supplied adapters.
    pub async fn run<DB>(&self, db: &DB, input: &SyncInputs) -> Result<SyncOutcome, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.status.send("Fetching latest block").await?;
        let latest_block = self.events.latest_block().await?;

        self.status.send("Running bootstrap").await?;
        self.bootstrap
            .engine_run(
                db,
                &BootstrapConfig {
                    target_key: input.target.clone(),
                    dump_stmt: input.dump_str.as_ref().map(SqlStatement::new),
                    latest_block,
                },
            )
            .await?;

        self.status.send("Computing sync window").await?;
        let (start_block, target_block) = self
            .window
            .compute(db, &input.target, &input.cfg, latest_block)
            .await?;
        if start_block > target_block {
            self.status.send("No work for current window").await?;
            return Ok(SyncOutcome {
                target: input.target.clone(),
                start_block,
                target_block,
                fetched_logs: 0,
                decoded_events: 0,
            });
        }

        self.status.send("Fetching orderbook logs").await?;
        let orderbook_logs = self
            .events
            .fetch_orderbook(
                input.target.orderbook_address,
                start_block,
                target_block,
                &input.cfg.fetch,
            )
            .await?;

        self.status.send("Decoding orderbook logs").await?;
        let mut decoded_events = self.events.decode(&orderbook_logs)?;

        let mut all_raw_logs = orderbook_logs.clone();

        // Collect store addresses from decoded orderbook events and previously known stores.
        let mut store_addresses = collect_store_addresses(&decoded_events);
        let existing_store_rows = load_known_store_addresses(db, &input.target).await?;
        for row in existing_store_rows {
            if !row.store_address.is_empty() {
                store_addresses.insert(row.store_address);
            }
        }

        // Collect token addresses from decoded orderbook events.
        let token_addresses: BTreeSet<Address> = collect_token_addresses(&decoded_events);
        let token_addresses_vec: Vec<Address> = token_addresses.iter().copied().collect();

        // Run token lookup and store log fetch in parallel.
        let tokens_fut = self.tokens.load_existing(
            db,
            input.target.chain_id,
            input.target.orderbook_address,
            &token_addresses_vec,
        );

        let store_fetch_fut = async {
            if store_addresses.is_empty() {
                Ok::<(Vec<LogEntryResponse>, Vec<DecodedEventData<DecodedEvent>>), LocalDbError>((
                    Vec::new(),
                    Vec::new(),
                ))
            } else {
                self.status.send("Fetching interpreter store logs").await?;
                let addresses = store_addresses.into_iter().collect::<Vec<Address>>();
                let logs = self
                    .events
                    .fetch_stores(&addresses, start_block, target_block, &input.cfg.fetch)
                    .await?;
                self.status.send("Decoding interpreter store logs").await?;
                let decoded = self.events.decode(&logs)?;
                Ok((logs, decoded))
            }
        };

        let (existing_tokens_res, store_pair_res) = tokio::join!(tokens_fut, store_fetch_fut);
        let existing_tokens = existing_tokens_res?;
        let (store_logs, mut decoded_store_events) = store_pair_res?;

        if !store_logs.is_empty() {
            all_raw_logs.extend(store_logs);
            decoded_events.append(&mut decoded_store_events);
            sort_decoded_events_by_block_and_log(&mut decoded_events)?;
        }

        let existing_set = existing_tokens
            .iter()
            .map(|row| row.token_address)
            .collect::<HashSet<Address>>();
        let missing_tokens = token_addresses
            .iter()
            .copied()
            .filter(|addr| !existing_set.contains(addr))
            .collect::<Vec<Address>>();
        let mut tokens_to_upsert = Vec::new();
        if !missing_tokens.is_empty() {
            self.status.send("Fetching missing token metadata").await?;
            tokens_to_upsert = self
                .tokens
                .fetch_missing(&input.metadata_rpcs, missing_tokens, &input.cfg.fetch)
                .await?;
        }

        self.status.send("Building SQL batch").await?;
        let batch = self.apply.build_batch(
            &input.target,
            target_block,
            &all_raw_logs,
            &decoded_events,
            &existing_tokens,
            &tokens_to_upsert,
        )?;

        self.status.send("Persisting to database").await?;
        self.apply.persist(db, &batch).await?;

        self.status.send("Running post-sync export").await?;
        self.apply
            .export_dump(db, &input.target, target_block)
            .await?;

        Ok(SyncOutcome {
            target: input.target.clone(),
            start_block,
            target_block,
            fetched_logs: all_raw_logs.len(),
            decoded_events: decoded_events.len(),
        })
    }
}

async fn load_known_store_addresses<DB>(
    db: &DB,
    target: &TargetKey,
) -> Result<Vec<StoreAddressRow>, LocalDbError>
where
    DB: LocalDbQueryExecutor + ?Sized,
{
    db.query_json(&fetch_store_addresses_stmt(
        target.chain_id,
        target.orderbook_address,
    ))
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::TargetKey;
    use crate::local_db::query::{
        fetch_store_addresses::StoreAddressRow, LocalDbQueryError, SqlStatement, SqlStatementBatch,
        SqlValue,
    };
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use std::cell::RefCell;

    struct MockExecutor {
        response: RefCell<Result<Vec<StoreAddressRow>, LocalDbQueryError>>,
        statements: RefCell<Vec<SqlStatement>>,
    }

    impl MockExecutor {
        fn with_response(response: Result<Vec<StoreAddressRow>, LocalDbQueryError>) -> Self {
            Self {
                response: RefCell::new(response),
                statements: RefCell::new(Vec::new()),
            }
        }

        fn success(rows: Vec<StoreAddressRow>) -> Self {
            Self::with_response(Ok(rows))
        }

        fn failure(err: LocalDbQueryError) -> Self {
            Self::with_response(Err(err))
        }

        fn recorded(&self) -> Vec<SqlStatement> {
            self.statements.borrow().clone()
        }
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for MockExecutor {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            panic!("execute_batch should not be called");
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: crate::local_db::query::FromDbJson,
        {
            self.statements.borrow_mut().push(stmt.clone());
            match self.response.borrow().clone() {
                Ok(rows) => {
                    let value = serde_json::to_value(rows).expect("serialize rows");
                    serde_json::from_value(value)
                        .map_err(|err| LocalDbQueryError::deserialization(err.to_string()))
                }
                Err(err) => Err(err),
            }
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            panic!("query_text should not be called");
        }
    }

    #[tokio::test]
    async fn load_known_store_addresses_yields_rows_and_binds_params() {
        let rows = vec![
            StoreAddressRow {
                store_address: Address::from([0x11u8; 20]),
            },
            StoreAddressRow {
                store_address: Address::from([0x22u8; 20]),
            },
        ];
        let executor = MockExecutor::success(rows.clone());
        let target = TargetKey {
            chain_id: 42161,
            orderbook_address: Address::repeat_byte(0xAA),
        };

        let actual = load_known_store_addresses(&executor, &target)
            .await
            .expect("fetch rows");

        assert_eq!(actual, rows);
        let calls = executor.recorded();
        assert_eq!(calls.len(), 1);
        let stmt = &calls[0];
        assert_eq!(stmt.params().len(), 2);
        assert_eq!(stmt.params()[0], SqlValue::U64(target.chain_id as u64));
        assert_eq!(
            stmt.params()[1],
            SqlValue::Text(target.orderbook_address.to_string())
        );
    }

    #[tokio::test]
    async fn load_known_store_addresses_returns_empty_vec() {
        let executor = MockExecutor::success(Vec::new());
        let target = TargetKey {
            chain_id: 10,
            orderbook_address: Address::repeat_byte(0xBB),
        };

        let actual = load_known_store_addresses(&executor, &target)
            .await
            .expect("fetch rows");
        assert!(actual.is_empty());
    }

    #[tokio::test]
    async fn load_known_store_addresses_propagates_query_error() {
        let executor = MockExecutor::failure(LocalDbQueryError::database("boom"));
        let target = TargetKey {
            chain_id: 1,
            orderbook_address: Address::repeat_byte(0xCC),
        };

        let err = load_known_store_addresses(&executor, &target)
            .await
            .expect_err("should propagate error");
        match err {
            LocalDbError::LocalDbQueryError(inner) => match inner {
                LocalDbQueryError::Database { message } => assert_eq!(message, "boom"),
                other => panic!("unexpected query error: {other:?}"),
            },
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
