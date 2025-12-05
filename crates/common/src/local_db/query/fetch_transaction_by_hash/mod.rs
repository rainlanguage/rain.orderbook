use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

/// Transaction info returned from local DB query
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbTransaction {
    pub transaction_hash: B256,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub owner: Address,
}

/// Builds a SQL statement to fetch transaction info by transaction hash
/// from the vault_balance_changes table.
pub fn build_fetch_transaction_by_hash_stmt(
    ob_id: &OrderbookIdentifier,
    tx_hash: B256,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        vec![
            SqlValue::from(ob_id.chain_id),
            SqlValue::from(ob_id.orderbook_address),
            SqlValue::from(tx_hash),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256};

    #[test]
    fn builds_correct_sql_with_params() {
        let orderbook = address!("0x1234567890123456789012345678901234567890");
        let tx_hash = b256!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
        let ob_id = OrderbookIdentifier::new(1, orderbook);

        let stmt = build_fetch_transaction_by_hash_stmt(&ob_id, tx_hash);

        assert!(stmt.sql.contains("SELECT"));
        assert!(stmt.sql.contains("FROM vault_balance_changes"));
        assert!(stmt.sql.contains("transaction_hash"));
        assert_eq!(stmt.params.len(), 3);
    }
}
