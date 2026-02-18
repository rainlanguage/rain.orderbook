use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::Address;

const QUERY_TEMPLATE: &str = include_str!("query.sql");
const DEFAULT_PAGE_SIZE: u16 = 100;

pub fn build_fetch_owner_trades_stmt(
    ob_id: &OrderbookIdentifier,
    owner: Address,
    page: Option<u16>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));
    stmt.push(SqlValue::from(owner));

    let page_num = page.unwrap_or(1).max(1);
    let limit = DEFAULT_PAGE_SIZE as i64;
    let offset = (page_num as i64 - 1) * limit;
    stmt.push(SqlValue::I64(limit));
    stmt.push(SqlValue::I64(offset));

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn builds_with_default_page() {
        let owner = Address::repeat_byte(0xaa);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(42161, Address::ZERO),
            owner,
            None,
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[0], SqlValue::U64(42161));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert!(
            matches!(&stmt.params[2], SqlValue::Text(s) if s.to_lowercase() == owner.to_string().to_lowercase())
        );
        assert_eq!(stmt.params[3], SqlValue::I64(100));
        assert_eq!(stmt.params[4], SqlValue::I64(0));
    }

    #[test]
    fn builds_with_page_2() {
        let owner = Address::repeat_byte(0xbb);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(1, Address::ZERO),
            owner,
            Some(2),
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[3], SqlValue::I64(100));
        assert_eq!(stmt.params[4], SqlValue::I64(100));
    }
}
