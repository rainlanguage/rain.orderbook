use crate::local_db::query::{SqlStatement, SqlValue};
use alloy::{hex, primitives::Address};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn build_update_last_synced_block_stmt(
    chain_id: u32,
    orderbook_address: Address,
    block_number: u64,
) -> SqlStatement {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    // ?1 -> chain id
    stmt.push(SqlValue::I64(chain_id as i64));
    // ?2 -> orderbook address
    stmt.push(SqlValue::Text(hex::encode_prefixed(orderbook_address)));
    // ?3 -> block number
    stmt.push(SqlValue::I64(block_number as i64));
    stmt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_update_query() {
        let stmt = build_update_last_synced_block_stmt(42161, Address::from([0x11u8; 20]), 999);
        assert!(stmt.sql.contains("ON CONFLICT"));
        assert_eq!(stmt.params.len(), 3);
        assert_eq!(stmt.params[0], SqlValue::I64(42161));
        assert_eq!(
            stmt.params[1],
            SqlValue::Text(hex::encode_prefixed(Address::from([0x11u8; 20])))
        );
        assert_eq!(stmt.params[2], SqlValue::I64(999));
    }
}
