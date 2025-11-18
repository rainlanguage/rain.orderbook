use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbVaultBalanceChange {
    pub transaction_hash: Bytes,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub owner: Address,
    pub change_type: String,
    pub token: Address,
    pub vault_id: U256,
    pub delta: String,
    pub running_balance: String,
}

pub fn build_fetch_balance_changes_stmt(
    ob_id: &OrderbookIdentifier,
    vault_id: U256,
    token: Address,
    owner: Address,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::from(ob_id.chain_id),
            SqlValue::from(ob_id.orderbook_address),
            SqlValue::from(vault_id),
            SqlValue::from(token),
            SqlValue::from(owner),
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
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
        );
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("?1 AS chain_id"));
        assert_eq!(stmt.params.len(), 5);
    }
}
