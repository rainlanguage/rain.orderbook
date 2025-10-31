use crate::erc20::TokenInfo;
use crate::local_db::pipeline::TokensPipeline;
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
        chain_id: u32,
        orderbook_address: Address,
        token_addrs_lower: &[Address],
    ) -> Result<Vec<Erc20TokenRow>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let Some(stmt) = build_fetch_stmt(chain_id, orderbook_address, token_addrs_lower)? else {
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
    use alloy::hex;
    use serde_json::json;

    struct MockDb {
        // Preloaded rows to return for query_json
        rows: Vec<Erc20TokenRow>,
        // Assertions for the incoming statement
        expect_in_clause: bool,
        expect_chain_id: Option<i64>,
        expect_orderbook: Option<String>,
        expect_addr_count: Option<usize>,
        // Optional explicit expected address parameter values (in order)
        expect_addr_values: Option<Vec<String>>,
        // Inject a DB error from query_json when set
        fail_query: bool,
        // Force a deserialization error from query_json when set
        return_malformed_json: bool,
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
            if self.fail_query {
                return Err(LocalDbQueryError::database("boom"));
            }
            if self.return_malformed_json {
                // Return a shape that won't deserialize to the expected T
                let bad = json!({"unexpected": "shape"});
                return serde_json::from_value::<T>(bad)
                    .map_err(|e| LocalDbQueryError::deserialization(e.to_string()));
            }
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
            if let Some(expected) = &self.expect_addr_values {
                let addr_params: Vec<String> = stmt
                    .params()
                    .iter()
                    .skip(2)
                    .map(|v| match v {
                        crate::local_db::query::SqlValue::Text(s) => s.clone(),
                        other => panic!("expected Text param for address, got {other:?}"),
                    })
                    .collect();
                assert_eq!(
                    &addr_params, expected,
                    "address params must match input order"
                );
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
            expect_addr_values: None,
            fail_query: false,
            return_malformed_json: false,
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
        let other_addr = Address::from([0xad; 20]);
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
            expect_addr_values: Some(vec![
                hex::encode_prefixed(token_addr),
                hex::encode_prefixed(other_addr),
            ]),
            fail_query: false,
            return_malformed_json: false,
        };
        let pipeline = DefaultTokensPipeline::new();
        let addrs = vec![token_addr, other_addr];
        let out = pipeline
            .load_existing(&db, 137, orderbook, &addrs)
            .await
            .expect("ok");
        assert_eq!(out, vec![row]);
    }

    #[tokio::test]
    async fn load_existing_propagates_db_error() {
        let orderbook = Address::from([0xcd; 20]);
        let token_addr = Address::from([0xae; 20]);
        let db = MockDb {
            rows: vec![],
            expect_in_clause: true,
            expect_chain_id: Some(1),
            expect_orderbook: Some(orderbook.to_string()),
            expect_addr_count: Some(1),
            expect_addr_values: Some(vec![hex::encode_prefixed(token_addr)]),
            fail_query: true,
            return_malformed_json: false,
        };
        let pipeline = DefaultTokensPipeline::new();
        let err = pipeline
            .load_existing(&db, 1, orderbook, &[token_addr])
            .await
            .expect_err("should fail");
        match err {
            LocalDbError::LocalDbQueryError(e) => match e {
                crate::local_db::query::LocalDbQueryError::Database { .. } => {}
                other => panic!("expected Database error, got {other:?}"),
            },
            other => panic!("expected LocalDbQueryError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn load_existing_deserialization_error_bubbles() {
        let orderbook = Address::from([0xde; 20]);
        let token_addr = Address::from([0xaf; 20]);
        let db = MockDb {
            rows: vec![],
            expect_in_clause: true,
            expect_chain_id: Some(1),
            expect_orderbook: Some(orderbook.to_string()),
            expect_addr_count: Some(1),
            expect_addr_values: Some(vec![hex::encode_prefixed(token_addr)]),
            fail_query: false,
            return_malformed_json: true,
        };
        let pipeline = DefaultTokensPipeline::new();
        let err = pipeline
            .load_existing(&db, 1, orderbook, &[token_addr])
            .await
            .expect_err("should fail");
        match err {
            LocalDbError::LocalDbQueryError(e) => match e {
                crate::local_db::query::LocalDbQueryError::Deserialization { .. } => {}
                other => panic!("expected Deserialization error, got {other:?}"),
            },
            other => panic!("expected LocalDbQueryError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn load_existing_keeps_duplicate_addresses_in_order() {
        let orderbook = Address::from([0xef; 20]);
        let addr_a = Address::from([0xaa; 20]);
        let addr_b = Address::from([0xbb; 20]);
        let db = MockDb {
            rows: vec![],
            expect_in_clause: true,
            expect_chain_id: Some(137),
            expect_orderbook: Some(orderbook.to_string()),
            expect_addr_count: Some(3),
            expect_addr_values: Some(vec![
                hex::encode_prefixed(addr_a),
                hex::encode_prefixed(addr_a),
                hex::encode_prefixed(addr_b),
            ]),
            fail_query: false,
            return_malformed_json: false,
        };
        let pipeline = DefaultTokensPipeline::new();
        let addrs = vec![addr_a, addr_a, addr_b];
        // We only care that params keep duplicates in order; rows are empty
        let out = pipeline
            .load_existing(&db, 137, orderbook, &addrs)
            .await
            .expect("ok");
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn fetch_missing_empty_returns_empty() {
        let pipeline = DefaultTokensPipeline::new();
        // No RPCs needed since missing is empty
        let out = pipeline
            .fetch_missing(&[], Vec::new(), &FetchConfig::default())
            .await
            .expect("ok");
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn fetch_missing_with_empty_rpcs_and_nonempty_missing_errors() {
        let pipeline = DefaultTokensPipeline::new();
        // With no RPCs and at least one missing address, expect an error
        let res = pipeline
            .fetch_missing(&[], vec![Address::ZERO], &FetchConfig::default())
            .await;
        assert!(res.is_err(), "expected error when RPCs are empty");
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

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn fetch_missing_multiple_tokens() {
            let local_evm = LocalEvm::new_with_tokens(2).await;
            let url = Url::parse(&local_evm.url()).unwrap();
            let t0 = local_evm.tokens[0].clone();
            let t1 = local_evm.tokens[1].clone();
            let a0: Address = *t0.address();
            let a1: Address = *t1.address();

            let pipeline = DefaultTokensPipeline::new();
            let mut out = pipeline
                .fetch_missing(&[url], vec![a0, a1], &FetchConfig::default())
                .await
                .expect("fetch ok");
            out.sort_by_key(|(a, _)| *a);
            let mut expected = vec![a0, a1];
            expected.sort();
            assert_eq!(out.len(), 2);
            assert_eq!(out.iter().map(|(a, _)| *a).collect::<Vec<_>>(), expected);
        }

        #[tokio::test]
        async fn fetch_missing_failure_bubbles_error() {
            let bad_url = Url::parse("http://127.0.0.1:1").unwrap();
            let pipeline = DefaultTokensPipeline::new();
            let err = pipeline
                .fetch_missing(&[bad_url], vec![Address::ZERO], &FetchConfig::default())
                .await
                .expect_err("expected error");
            match err {
                LocalDbError::ERC20Error(crate::erc20::Error::MulticallError(multicall_err)) => {
                    use alloy::providers::MulticallError;
                    assert!(
                        matches!(multicall_err, MulticallError::TransportError(_)),
                        "expected transport-related failure, got {multicall_err:?}"
                    );
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn fetch_missing_mixed_success_and_failure_returns_error() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let url = Url::parse(&local_evm.url()).unwrap();
            let good = *local_evm.tokens[0].address();
            let bad = Address::ZERO; // not a valid token contract in the fixture

            let pipeline = DefaultTokensPipeline::new();
            let err = pipeline
                .fetch_missing(&[url], vec![good, bad], &FetchConfig::default())
                .await
                .expect_err("expected overall error on partial failure");

            match err {
                LocalDbError::ERC20Error(crate::erc20::Error::MulticallError(multicall_err)) => {
                    use alloy::providers::MulticallError;
                    assert!(
                        matches!(multicall_err, MulticallError::DecodeError(_)),
                        "expected decode failure for invalid token address, got {multicall_err:?}"
                    );
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
        }
    }
}
