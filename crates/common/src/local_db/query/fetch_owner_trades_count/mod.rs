use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::Address;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub use crate::local_db::query::fetch_order_trades_count::{
    extract_trade_count, LocalDbTradeCountRow,
};

pub fn build_fetch_owner_trades_count_stmt(
    ob_id: &OrderbookIdentifier,
    owner: Address,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));
    stmt.push(SqlValue::from(owner));
    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn builds_stmt() {
        let owner = Address::repeat_byte(0xcc);
        let stmt = build_fetch_owner_trades_count_stmt(
            &OrderbookIdentifier::new(42161, Address::ZERO),
            owner,
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 3);
        assert_eq!(stmt.params[0], SqlValue::U64(42161));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert!(
            matches!(&stmt.params[2], SqlValue::Text(s) if s.to_lowercase() == owner.to_string().to_lowercase())
        );
    }
}
