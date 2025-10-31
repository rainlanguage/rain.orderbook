use crate::local_db::query::{SqlStatement, SqlValue};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: Address,
}

pub fn fetch_store_addresses_stmt(chain_id: u32, orderbook_address: Address) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_STORE_ADDRESSES_SQL,
        [
            SqlValue::from(chain_id as u64),
            SqlValue::from(orderbook_address.to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_binds_chain_and_orderbook() {
        let stmt = fetch_store_addresses_stmt(1, Address::ZERO);
        assert_eq!(stmt.sql, FETCH_STORE_ADDRESSES_SQL);
        assert_eq!(stmt.params.len(), 2);
        assert!(stmt
            .sql
            .to_lowercase()
            .contains("select distinct lower(store_address)"));
    }
}
