use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVaultBalanceChange {
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "log_index")]
    pub log_index: u64,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: u64,
    pub owner: String,
    #[serde(alias = "change_type")]
    pub change_type: String,
    pub token: String,
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub delta: String,
    #[serde(alias = "running_balance")]
    pub running_balance: String,
}

pub fn build_fetch_balance_changes_stmt(
    ob_id: &OrderbookIdentifier,
    vault_id: &str,
    token: &str,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::I64(ob_id.chain_id as i64),
            SqlValue::Text(ob_id.orderbook_address.to_string()),
            SqlValue::Text(vault_id.trim().to_string()),
            SqlValue::Text(token.trim().to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;

    use super::*;

    #[test]
    fn builds_with_params() {
        let stmt = build_fetch_balance_changes_stmt(
            &OrderbookIdentifier::new(1, Address::ZERO),
            "v01",
            "0xtoken",
        );
        assert!(stmt.sql.contains("lower(?3)  AS vault_id"));
        assert!(stmt.sql.contains("lower(?4)  AS token"));
        assert_eq!(stmt.params.len(), 4);
    }
}
