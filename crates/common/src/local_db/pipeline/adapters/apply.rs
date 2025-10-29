use crate::erc20::TokenInfo;
use crate::local_db::decode::DecodedEvent;
use crate::local_db::insert::{
    decoded_events_to_statements as build_decoded_event_sql,
    generate_erc20_token_statements as build_token_upserts,
    raw_events_to_statements as build_raw_event_sql,
};
use crate::local_db::pipeline::traits::{ApplyPipeline, TargetKey};
use crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow;
use crate::local_db::query::update_last_synced_block::build_update_last_synced_block_stmt;
use crate::local_db::query::{LocalDbQueryExecutor, SqlStatementBatch};
use crate::local_db::{decode::DecodedEventData, LocalDbError};
use crate::rpc_client::LogEntryResponse;
use alloy::primitives::Address;
use async_trait::async_trait;
use std::collections::HashMap;

/// Default implementation of the ApplyPipeline.
///
/// - Translates fetched/decoded inputs into SQL using existing builders.
/// - Wraps statements in a transaction and persists via the shared executor.
/// - Export hook is a no-op; producers can override in their adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultApplyPipeline;

impl DefaultApplyPipeline {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl ApplyPipeline for DefaultApplyPipeline {
    fn build_batch(
        &self,
        target: &TargetKey,
        target_block: u64,
        raw_logs: &[LogEntryResponse],
        decoded_events: &[DecodedEventData<DecodedEvent>],
        existing_tokens: &[Erc20TokenRow],
        tokens_to_upsert: &[(Address, TokenInfo)],
    ) -> Result<SqlStatementBatch, LocalDbError> {
        // 1) Build decimals map from existing rows and the incoming upserts
        let decimals_by_token: HashMap<Address, u8> = existing_tokens
            .iter()
            .map(|row| (row.token_address, row.decimals))
            .chain(
                tokens_to_upsert
                    .iter()
                    .map(|(addr, info)| (*addr, info.decimals)),
            )
            .collect();

        // 2) Build component batches
        let mut batch = SqlStatementBatch::new();

        // Raw events first
        let raw_batch = build_raw_event_sql(target.chain_id, target.orderbook_address, raw_logs)?;
        batch.extend(raw_batch);

        // Token upserts for the missing set only
        if !tokens_to_upsert.is_empty() {
            let upserts =
                build_token_upserts(target.chain_id, target.orderbook_address, tokens_to_upsert);
            batch.extend(upserts);
        }

        // Decoded orderbook/store events
        let decoded_batch = build_decoded_event_sql(
            target.chain_id,
            target.orderbook_address,
            decoded_events,
            &decimals_by_token,
        )?;
        batch.extend(decoded_batch);

        // Watermark update to target block
        batch.add(build_update_last_synced_block_stmt(
            target.chain_id,
            target.orderbook_address,
            target_block,
        ));

        // Ensure atomicity
        Ok(batch.ensure_transaction())
    }

    async fn persist<DB>(&self, db: &DB, batch: &SqlStatementBatch) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let stmts = batch.clone().ensure_transaction();
        db.execute_batch(&stmts).await.map_err(LocalDbError::from)
    }

    // export_dump: default no-op via trait default
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::{LocalDbQueryError, SqlStatement};

    struct MockDb {
        // capture executed SQL for assertions
        pub executed: std::sync::Mutex<Vec<String>>,
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            let mut exec = self.executed.lock().unwrap();
            for s in batch.statements() {
                exec.push(s.sql().to_string());
            }
            Ok(())
        }

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: crate::local_db::query::FromDbJson,
        {
            Err(LocalDbQueryError::database("not used"))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok(String::new())
        }
    }

    struct FailingDb;

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for FailingDb {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Err(LocalDbQueryError::database("boom"))
        }

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: crate::local_db::query::FromDbJson,
        {
            Err(LocalDbQueryError::database("not used"))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Err(LocalDbQueryError::database("not used"))
        }
    }

    fn sample_target() -> TargetKey {
        TargetKey {
            chain_id: 137,
            orderbook_address: Address::from([0u8; 20]),
        }
    }

    fn deposit_event(addr: Address) -> DecodedEventData<DecodedEvent> {
        use crate::local_db::decode::EventType;
        use alloy::primitives::U256;
        use rain_orderbook_bindings::IOrderBookV5::DepositV2;
        DecodedEventData {
            event_type: EventType::DepositV2,
            block_number: "0x1".into(),
            block_timestamp: "0x2".into(),
            transaction_hash: "0xabc".into(),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                sender: Address::from([1u8; 20]),
                token: addr,
                vaultId: U256::from(1u64).into(),
                depositAmountUint256: U256::from(1000u64),
            })),
        }
    }

    fn withdraw_event(addr: Address) -> DecodedEventData<DecodedEvent> {
        use crate::local_db::decode::EventType;
        use alloy::primitives::U256;
        use rain_orderbook_bindings::IOrderBookV5::WithdrawV2;
        DecodedEventData {
            event_type: EventType::WithdrawV2,
            block_number: "0x10".into(),
            block_timestamp: "0x20".into(),
            transaction_hash: "0xdef".into(),
            log_index: "0x1".into(),
            decoded_data: DecodedEvent::WithdrawV2(Box::new(WithdrawV2 {
                sender: Address::from([2u8; 20]),
                token: addr,
                vaultId: U256::from(2u64).into(),
                targetAmount: U256::from(2000u64).into(),
                withdrawAmount: U256::from(1500u64).into(),
                withdrawAmountUint256: U256::from(1500u64),
            })),
        }
    }

    #[test]
    fn build_batch_wraps_transaction_and_contains_watermark() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([3u8; 20]);

        // existing token with decimals 18
        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token,
            name: "Token".into(),
            symbol: "TKN".into(),
            decimals: 18,
        }];

        let decoded = vec![deposit_event(token)];

        let batch = pipeline
            .build_batch(&target, 123, &[], &decoded, &existing, &[])
            .expect("batch ok");

        assert!(
            batch.is_transaction(),
            "batch must be wrapped in a transaction"
        );

        let texts: Vec<_> = batch.statements().iter().map(|s| s.sql()).collect();
        assert!(
            texts.iter().any(|s| s.contains("INSERT INTO sync_status")),
            "watermark update present"
        );
    }

    #[tokio::test]
    async fn persist_wraps_when_needed() {
        let pipeline = DefaultApplyPipeline::new();
        let db = MockDb {
            executed: Default::default(),
        };
        // Build a simple non-transactional batch manually
        let mut batch = SqlStatementBatch::new();
        batch.add(crate::local_db::query::SqlStatement::new(
            "INSERT INTO foo VALUES (1)",
        ));

        // Should not error; persist will wrap
        pipeline.persist(&db, &batch).await.expect("persist ok");
        let executed = db.executed.lock().unwrap().clone();
        assert!(executed
            .first()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("BEGIN TRANSACTION")));
        assert!(executed
            .last()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("COMMIT")));
    }

    #[tokio::test]
    async fn persist_propagates_db_error() {
        let pipeline = DefaultApplyPipeline::new();
        let db = FailingDb;

        let mut batch = SqlStatementBatch::new();
        batch.add(crate::local_db::query::SqlStatement::new(
            "INSERT INTO foo VALUES (1)",
        ));

        let err = pipeline.persist(&db, &batch).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(e) => {
                let msg = format!("{}", e);
                assert!(msg.contains("Database operation failed"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn empty_work_window_only_watermark() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        let batch = pipeline
            .build_batch(&target, 42, &[], &[], &[], &[])
            .expect("batch ok");

        // Expect exactly BEGIN, UPDATE, COMMIT
        let texts: Vec<_> = batch.statements().iter().map(|s| s.sql().trim()).collect();
        assert_eq!(texts.first().copied(), Some("BEGIN TRANSACTION"));
        assert!(texts.iter().any(|s| s.contains("INSERT INTO sync_status")));
        assert_eq!(
            texts.last().copied().map(|s| s.to_ascii_uppercase()),
            Some("COMMIT".into())
        );
        assert!(batch.is_transaction());
    }

    #[test]
    fn decimals_from_upsert_override_existing() {
        use alloy::primitives::U256;
        use rain_math_float::Float;

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([7u8; 20]);

        // Existing says 6 decimals
        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token,
            name: "Token".into(),
            symbol: "T6".into(),
            decimals: 6,
        }];
        // Upsert says 18 decimals (override)
        let upserts = vec![(
            token,
            TokenInfo {
                name: "Token".into(),
                symbol: "T18".into(),
                decimals: 18,
            },
        )];

        let decoded = vec![deposit_event(token)];
        let batch = pipeline
            .build_batch(&target, 100, &[], &decoded, &existing, &upserts)
            .expect("batch ok");

        // Find the deposit INSERT and inspect params; ?10 is deposit_amount
        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO deposits"))
            .expect("deposit insert present");

        // Params: [chain_id, orderbook, block_number, block_timestamp, tx_hash, log_index, sender, token, vault_id, deposit_amount, deposit_amount_uint256]
        let deposit_amount_param = stmt.params().get(9).expect("param 10 present");
        let expected = Float::from_fixed_decimal(U256::from(1000u64), 18)
            .unwrap()
            .as_hex();

        match deposit_amount_param {
            crate::local_db::query::SqlValue::Text(val) => {
                assert_eq!(
                    val, &expected,
                    "upsert decimals should take precedence over existing row"
                );
            }
            other => panic!("unexpected param type for deposit_amount: {other:?}"),
        }
    }

    #[test]
    fn invalid_existing_token_address_is_ignored() {
        use alloy::primitives::U256;
        use rain_math_float::Float;

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([10u8; 20]);

        // Existing has an invalid address string
        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: Address::ZERO,
            name: "Bad".into(),
            symbol: "BAD".into(),
            decimals: 6,
        }];
        // Upsert provides correct decimals
        let upserts = vec![(
            token,
            TokenInfo {
                name: "Good".into(),
                symbol: "GOOD".into(),
                decimals: 9,
            },
        )];

        let decoded = vec![deposit_event(token)];
        let batch = pipeline
            .build_batch(&target, 100, &[], &decoded, &existing, &upserts)
            .expect("batch ok");

        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO deposits"))
            .expect("deposit insert present");
        let deposit_amount_param = stmt.params().get(9).expect("param 10 present");
        let expected = Float::from_fixed_decimal(U256::from(1000u64), 9)
            .unwrap()
            .as_hex();
        match deposit_amount_param {
            crate::local_db::query::SqlValue::Text(val) => {
                assert_eq!(val, &expected);
            }
            other => panic!("unexpected param type for deposit_amount: {other:?}"),
        }
    }

    #[test]
    fn missing_decimals_yields_error() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([9u8; 20]);
        let decoded = vec![deposit_event(token)];

        let err = pipeline
            .build_batch(&target, 1, &[], &decoded, &[], &[])
            .unwrap_err();

        use crate::local_db::insert::InsertError;
        match err {
            LocalDbError::InsertError(InsertError::MissingTokenDecimals { token: t }) => {
                assert_eq!(t, format!("0x{:x}", token));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn token_upserts_generated_for_provided_tokens() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        // Existing has A; upserts contain B
        let token_a = Address::from([1u8; 20]);
        let token_b = Address::from([2u8; 20]);
        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token_a,
            name: "A".into(),
            symbol: "A".into(),
            decimals: 6,
        }];
        let upserts = vec![(
            token_b,
            TokenInfo {
                name: "B".into(),
                symbol: "B".into(),
                decimals: 8,
            },
        )];

        let batch = pipeline
            .build_batch(&target, 1, &[], &[], &existing, &upserts)
            .expect("batch ok");

        let upsert_stmts: Vec<_> = batch
            .statements()
            .iter()
            .filter(|s| s.sql().starts_with("INSERT INTO erc20_tokens"))
            .collect();
        assert_eq!(
            upsert_stmts.len(),
            1,
            "upsert statements are generated for provided tokens"
        );

        // The token param is ?3 in the token upsert builder
        let addr_param = upsert_stmts[0].params().get(2).unwrap();
        match addr_param {
            crate::local_db::query::SqlValue::Text(val) => {
                assert_eq!(val, &format!("0x{:x}", token_b));
            }
            other => panic!("unexpected param type for token: {other:?}"),
        }
    }

    #[test]
    fn watermark_param_matches_target_block() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        let target_block = 12345u64;
        let batch = pipeline
            .build_batch(&target, target_block, &[], &[], &[], &[])
            .expect("batch ok");

        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO sync_status"))
            .expect("watermark update present");

        match stmt.params().get(2) {
            Some(crate::local_db::query::SqlValue::I64(v)) => {
                assert_eq!(*v as u64, target_block);
            }
            Some(crate::local_db::query::SqlValue::U64(v)) => {
                assert_eq!(*v, target_block);
            }
            other => panic!("unexpected watermark param: {other:?}"),
        }
    }

    #[test]
    fn decimals_from_existing_when_no_upserts() {
        use alloy::primitives::U256;
        use rain_math_float::Float;

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([13u8; 20]);

        // Only existing provides decimals
        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token,
            name: "E".into(),
            symbol: "E".into(),
            decimals: 9,
        }];

        let decoded = vec![deposit_event(token)];
        let batch = pipeline
            .build_batch(&target, 100, &[], &decoded, &existing, &[])
            .expect("batch ok");

        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO deposits"))
            .expect("deposit insert present");
        let deposit_amount_param = stmt.params().get(9).expect("param 10 present");
        let expected = Float::from_fixed_decimal(U256::from(1000u64), 9)
            .unwrap()
            .as_hex();
        match deposit_amount_param {
            crate::local_db::query::SqlValue::Text(val) => {
                assert_eq!(val, &expected);
            }
            other => panic!("unexpected param type for deposit_amount: {other:?}"),
        }
    }

    #[tokio::test]
    async fn persist_preserves_prewrapped_transaction() {
        let pipeline = DefaultApplyPipeline::new();
        let db = MockDb {
            executed: Default::default(),
        };

        // Build an already transactional batch
        let mut batch = SqlStatementBatch::new();
        batch.add(crate::local_db::query::SqlStatement::new(
            "INSERT INTO foo VALUES (1)",
        ));
        let wrapped = batch.ensure_transaction();

        pipeline.persist(&db, &wrapped).await.expect("persist ok");

        let executed = db.executed.lock().unwrap().clone();
        let begin_count = executed
            .iter()
            .filter(|s| s.trim().eq_ignore_ascii_case("BEGIN TRANSACTION"))
            .count();
        let commit_count = executed
            .iter()
            .filter(|s| s.trim().eq_ignore_ascii_case("COMMIT"))
            .count();
        assert_eq!(begin_count, 1, "no duplicate BEGIN statements");
        assert_eq!(commit_count, 1, "no duplicate COMMIT statements");
        assert!(executed
            .first()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("BEGIN TRANSACTION")));
        assert!(executed
            .last()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("COMMIT")));
    }

    #[test]
    fn duplicate_upserts_last_wins_for_decimals() {
        use alloy::primitives::U256;
        use rain_math_float::Float;

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([11u8; 20]);

        // Provide two upserts for same token with differing decimals; later should win
        let upserts = vec![
            (
                token,
                TokenInfo {
                    name: "Tok".into(),
                    symbol: "T".into(),
                    decimals: 6,
                },
            ),
            (
                token,
                TokenInfo {
                    name: "Tok".into(),
                    symbol: "T".into(),
                    decimals: 18,
                },
            ),
        ];

        let decoded = vec![deposit_event(token)];
        let batch = pipeline
            .build_batch(&target, 100, &[], &decoded, &[], &upserts)
            .expect("batch ok");

        // Two upserts present
        let upsert_count = batch
            .statements()
            .iter()
            .filter(|s| s.sql().starts_with("INSERT INTO erc20_tokens"))
            .count();
        assert_eq!(upsert_count, 2);

        // Deposit amount uses last upsert decimals (18)
        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO deposits"))
            .expect("deposit insert present");
        let deposit_amount_param = stmt.params().get(9).expect("param 10 present");
        let expected = Float::from_fixed_decimal(U256::from(1000u64), 18)
            .unwrap()
            .as_hex();
        match deposit_amount_param {
            crate::local_db::query::SqlValue::Text(val) => {
                assert_eq!(val, &expected);
            }
            other => panic!("unexpected param type for deposit_amount: {other:?}"),
        }
    }

    #[test]
    fn token_upserts_chain_id_param_matches_target() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        let token = Address::from([5u8; 20]);
        let upserts = vec![(
            token,
            TokenInfo {
                name: "X".into(),
                symbol: "X".into(),
                decimals: 18,
            },
        )];

        let batch = pipeline
            .build_batch(&target, 1, &[], &[], &[], &upserts)
            .expect("batch ok");

        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO erc20_tokens"))
            .expect("token upsert present");
        match stmt.params().first() {
            Some(crate::local_db::query::SqlValue::U64(chain)) => {
                assert_eq!(*chain, target.chain_id as u64);
            }
            Some(crate::local_db::query::SqlValue::I64(chain)) => {
                assert_eq!(*chain as u64, target.chain_id as u64);
            }
            other => panic!("unexpected chain_id param: {other:?}"),
        }
    }

    #[test]
    fn raw_events_sorted_by_block_then_log() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        // Two raw logs out of order
        let mk = |block: u64, log_index: u64| LogEntryResponse {
            address: "0x1111111111111111111111111111111111111111".into(),
            topics: vec![],
            data: "0x".into(),
            block_number: format!("0x{:x}", block),
            block_timestamp: Some("0x1".into()),
            transaction_hash: format!("0x{:x}", block),
            transaction_index: "0x0".into(),
            block_hash: format!("0x{:x}", block),
            log_index: format!("0x{:x}", log_index),
            removed: false,
        };
        let a = mk(10, 5);
        let b = mk(10, 3);

        let batch = pipeline
            .build_batch(&target, 10, &[a, b], &[], &[], &[])
            .expect("batch ok");

        let raws: Vec<_> = batch
            .statements()
            .iter()
            .filter(|s| s.sql().starts_with("INSERT INTO raw_events"))
            .collect();
        assert_eq!(raws.len(), 2);
        // Check param ?3 block_number and ?6 log_index are sorted ascending
        let get_u = |stmt: &SqlStatement, idx: usize| match stmt.params().get(idx) {
            Some(crate::local_db::query::SqlValue::U64(v)) => *v,
            Some(crate::local_db::query::SqlValue::I64(v)) => *v as u64,
            other => panic!("unexpected param type: {other:?}"),
        };
        let b1 = get_u(raws[0], 2);
        let l1 = get_u(raws[0], 5);
        let b2 = get_u(raws[1], 2);
        let l2 = get_u(raws[1], 5);
        assert!(b1 <= b2);
        if b1 == b2 {
            assert!(l1 < l2);
        }
    }

    #[test]
    fn only_raw_logs_emitted_when_no_tokens_or_decoded() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        let mk = |block: u64, log_index: u64| LogEntryResponse {
            address: "0x1111111111111111111111111111111111111111".into(),
            topics: vec![],
            data: "0x01".into(),
            block_number: format!("0x{:x}", block),
            block_timestamp: Some("0x1".into()),
            transaction_hash: format!("0x{:x}", block),
            transaction_index: "0x0".into(),
            block_hash: format!("0x{:x}", block),
            log_index: format!("0x{:x}", log_index),
            removed: false,
        };
        let raw = [mk(1, 0), mk(1, 1)];

        let batch = pipeline
            .build_batch(&target, 2, &raw, &[], &[], &[])
            .expect("batch ok");

        let texts: Vec<_> = batch.statements().iter().map(|s| s.sql()).collect();
        assert!(texts
            .iter()
            .any(|s| s.starts_with("INSERT INTO raw_events")));
        assert!(!texts
            .iter()
            .any(|s| s.starts_with("INSERT INTO erc20_tokens")));
        assert!(!texts.iter().any(|s| s.starts_with("INSERT INTO deposits")));
        assert!(texts.iter().any(|s| s.contains("INSERT INTO sync_status")));
    }

    #[test]
    fn watermark_emitted_exactly_once() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let batch = pipeline
            .build_batch(&target, 77, &[], &[], &[], &[])
            .expect("batch ok");
        let count = batch
            .statements()
            .iter()
            .filter(|s| s.sql().starts_with("INSERT INTO sync_status"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn statements_order_raw_then_tokens_then_decoded_then_watermark() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        // Build one raw, one token upsert, one decoded deposit
        let mk_raw = |block: u64, log_index: u64| LogEntryResponse {
            address: "0x1111111111111111111111111111111111111111".into(),
            topics: vec![],
            data: "0x02".into(),
            block_number: format!("0x{:x}", block),
            block_timestamp: Some("0x1".into()),
            transaction_hash: format!("0x{:x}", block),
            transaction_index: "0x0".into(),
            block_hash: format!("0x{:x}", block),
            log_index: format!("0x{:x}", log_index),
            removed: false,
        };
        let token = Address::from([8u8; 20]);
        let upserts = vec![(
            token,
            TokenInfo {
                name: "Y".into(),
                symbol: "Y".into(),
                decimals: 18,
            },
        )];
        let decoded = vec![deposit_event(token)];

        let batch = pipeline
            .build_batch(&target, 9, &[mk_raw(9, 0)], &decoded, &[], &upserts)
            .expect("batch ok");

        let idx_raw = batch
            .statements()
            .iter()
            .position(|s| s.sql().starts_with("INSERT INTO raw_events"))
            .expect("raw present");
        let idx_token = batch
            .statements()
            .iter()
            .position(|s| s.sql().starts_with("INSERT INTO erc20_tokens"))
            .expect("token upsert present");
        let idx_decoded = batch
            .statements()
            .iter()
            .position(|s| s.sql().starts_with("INSERT INTO deposits"))
            .expect("decoded present");
        let idx_watermark = batch
            .statements()
            .iter()
            .position(|s| s.sql().starts_with("INSERT INTO sync_status"))
            .expect("watermark present");
        let idx_commit = batch
            .statements()
            .iter()
            .position(|s| s.sql().trim().eq_ignore_ascii_case("COMMIT"))
            .expect("commit present");

        assert!(idx_raw < idx_token, "raw should precede token upserts");
        assert!(idx_token < idx_decoded, "token upserts before decoded");
        assert!(idx_decoded < idx_watermark, "decoded before watermark");
        assert!(idx_watermark < idx_commit, "watermark before commit");
    }

    #[test]
    fn deterministic_batch_shape_for_identical_inputs() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([3u8; 20]);

        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token,
            name: "Token".into(),
            symbol: "TKN".into(),
            decimals: 18,
        }];
        let decoded = vec![deposit_event(token)];

        let b1 = pipeline
            .build_batch(&target, 5, &[], &decoded, &existing, &[])
            .expect("b1 ok");
        let b2 = pipeline
            .build_batch(&target, 5, &[], &decoded, &existing, &[])
            .expect("b2 ok");

        assert_eq!(b1.statements().len(), b2.statements().len());
        for (s1, s2) in b1.statements().iter().zip(b2.statements().iter()) {
            assert_eq!(s1.sql(), s2.sql());
            assert_eq!(s1.params(), s2.params());
        }
    }

    #[test]
    fn includes_withdraw_decoded_events() {
        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([12u8; 20]);

        let batch = pipeline
            .build_batch(&target, 5, &[], &[withdraw_event(token)], &[], &[])
            .expect("batch ok");

        assert!(batch
            .statements()
            .iter()
            .any(|s| s.sql().starts_with("INSERT INTO withdrawals")));
    }

    #[test]
    fn multi_token_decimals_mapped_correctly() {
        use alloy::primitives::U256;
        use rain_math_float::Float;

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();

        // Two tokens: A gets decimals from existing, B from upserts
        let token_a = Address::from([21u8; 20]);
        let token_b = Address::from([22u8; 20]);

        let existing = vec![Erc20TokenRow {
            chain_id: target.chain_id as u32,
            orderbook_address: target.orderbook_address,
            token_address: token_a,
            name: "A".into(),
            symbol: "A".into(),
            decimals: 6,
        }];
        let upserts = vec![(
            token_b,
            TokenInfo {
                name: "B".into(),
                symbol: "B".into(),
                decimals: 9,
            },
        )];

        let decoded = vec![deposit_event(token_a), deposit_event(token_b)];
        let batch = pipeline
            .build_batch(&target, 100, &[], &decoded, &existing, &upserts)
            .expect("batch ok");

        // Collect deposit statements and check each token's deposit_amount uses the right decimals
        let deposits: Vec<_> = batch
            .statements()
            .iter()
            .filter(|s| s.sql().starts_with("INSERT INTO deposits"))
            .collect();
        assert_eq!(deposits.len(), 2, "expected two deposit inserts");

        for stmt in deposits {
            // token param is ?8 (index 7)
            let token_param = stmt.params().get(7).expect("token param present");
            let amount_param = stmt.params().get(9).expect("deposit_amount param present");

            match token_param {
                crate::local_db::query::SqlValue::Text(addr)
                    if addr == &format!("0x{:x}", token_a) =>
                {
                    let expected = Float::from_fixed_decimal(U256::from(1000u64), 6)
                        .unwrap()
                        .as_hex();
                    match amount_param {
                        crate::local_db::query::SqlValue::Text(val) => assert_eq!(val, &expected),
                        other => panic!("unexpected param type for deposit_amount: {other:?}"),
                    }
                }
                crate::local_db::query::SqlValue::Text(addr)
                    if addr == &format!("0x{:x}", token_b) =>
                {
                    let expected = Float::from_fixed_decimal(U256::from(1000u64), 9)
                        .unwrap()
                        .as_hex();
                    match amount_param {
                        crate::local_db::query::SqlValue::Text(val) => assert_eq!(val, &expected),
                        other => panic!("unexpected param type for deposit_amount: {other:?}"),
                    }
                }
                other => panic!("unexpected token param: {other:?}"),
            }
        }
    }

    #[test]
    fn withdraw_params_are_hex_and_uint256_is_32_bytes() {
        use alloy::{hex, primitives::U256};

        let pipeline = DefaultApplyPipeline::new();
        let target = sample_target();
        let token = Address::from([15u8; 20]);

        let batch = pipeline
            .build_batch(&target, 5, &[], &[withdraw_event(token)], &[], &[])
            .expect("batch ok");

        let stmt = batch
            .statements()
            .iter()
            .find(|s| s.sql().starts_with("INSERT INTO withdrawals"))
            .expect("withdraw insert present");

        // token param is ?8 (index 7)
        match stmt.params().get(7) {
            Some(crate::local_db::query::SqlValue::Text(v)) => {
                assert_eq!(v, &format!("0x{:x}", token))
            }
            other => panic!("unexpected token param: {other:?}"),
        }

        // withdraw_amount_uint256 is ?12 (index 11) and must be 32-byte hex of 1500
        let expected_uint256 = hex::encode_prefixed(U256::from(1500u64).to_be_bytes::<32>());
        match stmt.params().get(11) {
            Some(crate::local_db::query::SqlValue::Text(v)) => assert_eq!(v, &expected_uint256),
            other => panic!("unexpected withdraw_amount_uint256 param: {other:?}"),
        }
    }
}
