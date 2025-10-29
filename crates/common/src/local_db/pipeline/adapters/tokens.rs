use crate::erc20::TokenInfo;
use crate::local_db::pipeline::traits::TokensPipeline;
use crate::local_db::query::fetch_erc20_tokens_by_addresses::{build_fetch_stmt, Erc20TokenRow};
use crate::local_db::query::LocalDbQueryExecutor;
use crate::local_db::token_fetch::fetch_erc20_metadata_concurrent;
use crate::local_db::{FetchConfig, LocalDbError};
use alloy::primitives::Address;
use async_trait::async_trait;
use url::Url;

/// Default implementation of the TokensPipeline that delegates to the
/// existing concurrent ERC-20 metadata fetcher and query builders.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultTokensPipeline;

impl DefaultTokensPipeline {
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl TokensPipeline for DefaultTokensPipeline {
    async fn load_existing<DB>(
        &self,
        db: &DB,
        chain_id: u64,
        orderbook_address: Address,
        token_addrs_lower: &[Address],
    ) -> Result<Vec<Erc20TokenRow>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let Some(stmt) = build_fetch_stmt(chain_id as u32, orderbook_address, token_addrs_lower)?
        else {
            return Ok(vec![]);
        };
        let rows: Vec<Erc20TokenRow> = db.query_json(&stmt).await?;
        Ok(rows)
    }

    async fn fetch_missing(
        &self,
        rpcs: &[Url],
        missing: Vec<Address>,
        cfg: &FetchConfig,
    ) -> Result<Vec<(Address, TokenInfo)>, LocalDbError> {
        fetch_erc20_metadata_concurrent(rpcs.to_vec(), missing, cfg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::{LocalDbQueryError, SqlStatement, SqlStatementBatch};
    use serde_json::json;

    struct MockDb {
        // Preloaded rows to return for query_json
        rows: Vec<Erc20TokenRow>,
        // Assertions for the incoming statement
        expect_in_clause: bool,
        expect_chain_id: Option<i64>,
        expect_orderbook: Option<String>,
        expect_addr_count: Option<usize>,
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: crate::local_db::query::FromDbJson,
        {
            if let Some(cid) = self.expect_chain_id {
                match stmt.params().first() {
                    Some(crate::local_db::query::SqlValue::I64(v)) => assert_eq!(*v, cid),
                    other => panic!("expected first param I64({cid}), got {other:?}"),
                }
            }
            if let Some(expected_orderbook) = self.expect_orderbook.as_ref() {
                match stmt.params().get(1) {
                    Some(crate::local_db::query::SqlValue::Text(actual)) => {
                        assert_eq!(actual, expected_orderbook);
                    }
                    other => {
                        panic!("expected second param Text({expected_orderbook}), got {other:?}")
                    }
                }
            }
            if let Some(n) = self.expect_addr_count {
                // There should be chain id + orderbook + n address params
                assert_eq!(stmt.params().len(), 2 + n);
            }
            if self.expect_in_clause {
                assert!(stmt.sql().contains("IN ("), "expected IN clause in SQL");
            }
            let v = serde_json::to_value(&self.rows)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))?;
            serde_json::from_value::<T>(json!(v))
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok(String::new())
        }
    }

    #[tokio::test]
    async fn load_existing_empty_addresses_short_circuits() {
        let orderbook = Address::from([0xbe; 20]);
        let db = MockDb {
            rows: vec![],
            expect_in_clause: false,
            expect_chain_id: None,
            expect_orderbook: None,
            expect_addr_count: None,
        };
        let pipeline = DefaultTokensPipeline::new();
        let out = pipeline
            .load_existing(&db, 137, orderbook, &[])
            .await
            .expect("ok");
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn load_existing_builds_query_and_returns_rows() {
        let orderbook = Address::from([0xab; 20]);
        let token_addr = Address::from([0xac; 20]);
        let row = Erc20TokenRow {
            chain_id: 137,
            orderbook_address: orderbook,
            token_address: token_addr,
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
        };
        let db = MockDb {
            rows: vec![row.clone()],
            expect_in_clause: true,
            expect_chain_id: Some(137),
            expect_orderbook: Some(orderbook.to_string()),
            expect_addr_count: Some(2),
        };
        let pipeline = DefaultTokensPipeline::new();
        let addrs = vec![token_addr, Address::from([0xad; 20])];
        let out = pipeline
            .load_existing(&db, 137, orderbook, &addrs)
            .await
            .expect("ok");
        assert_eq!(out, vec![row]);
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use alloy::primitives::Address;
        use rain_orderbook_test_fixtures::LocalEvm;

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn fetch_missing_delegates_to_concurrent_fetcher() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let url = Url::parse(&local_evm.url()).unwrap();
            let token = local_evm.tokens[0].clone();
            let addr: Address = *token.address();

            let pipeline = DefaultTokensPipeline::new();
            let out = pipeline
                .fetch_missing(&[url], vec![addr], &FetchConfig::default())
                .await
                .expect("fetch ok");

            assert_eq!(out.len(), 1);
            assert_eq!(out[0].0, addr);
            assert_eq!(out[0].1.decimals, 18);
            assert_eq!(out[0].1.symbol, "TOKEN1");
            assert_eq!(out[0].1.name, "Token1");
        }
    }
}
