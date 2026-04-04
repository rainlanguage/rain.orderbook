use crate::local_db::{
    query::{fetch_order_trades::LocalDbOrderTrade, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::B256;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

/// Builds the SQL statement for retrieving all trades emitted by a transaction.
pub fn build_fetch_transaction_trades_stmt(
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

pub type LocalDbTransactionTrade = LocalDbOrderTrade;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256};

    #[test]
    fn builds_with_chain_id_orderbook_and_tx_hash() {
        let tx_hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let stmt = build_fetch_transaction_trades_stmt(
            &OrderbookIdentifier::new(8453, address!("0x1111111111111111111111111111111111111111")),
            tx_hash,
        );

        assert!(stmt.sql.contains("FROM take_orders"));
        assert!(stmt.sql.contains("FROM clear_v3_events"));
        assert_eq!(stmt.params.len(), 3);
    }
}
