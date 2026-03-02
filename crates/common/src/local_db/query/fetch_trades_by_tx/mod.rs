use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::B256;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn build_fetch_trades_by_tx_stmt(
    ob_id: &OrderbookIdentifier,
    tx_hash: B256,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));
    stmt.push(SqlValue::from(tx_hash));
    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{b256, Address},
    };

    #[test]
    fn builds_with_chain_id_and_tx_hash() {
        let tx_hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadface");
        let stmt =
            build_fetch_trades_by_tx_stmt(&OrderbookIdentifier::new(137, Address::ZERO), tx_hash)
                .unwrap();
        assert_eq!(stmt.params.len(), 3);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(hex::encode_prefixed(tx_hash))
        );
    }
}
