use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use serde::{Deserialize, Serialize};

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: Address,
}

pub fn fetch_store_addresses_stmt(ob_id: &OrderbookIdentifier) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_STORE_ADDRESSES_SQL,
        [
            SqlValue::from(ob_id.chain_id as u64),
            SqlValue::from(ob_id.orderbook_address.to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;

    use super::*;

    #[test]
    fn stmt_binds_chain_and_orderbook() {
        let stmt = fetch_store_addresses_stmt(&OrderbookIdentifier::new(1, Address::ZERO));
        assert_eq!(stmt.sql, FETCH_STORE_ADDRESSES_SQL);
        assert_eq!(stmt.params.len(), 2);
        assert!(stmt
            .sql
            .to_lowercase()
            .contains("select distinct lower(store_address)"));
    }
}
