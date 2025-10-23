use serde::{Deserialize, Serialize};

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: String,
}

pub fn fetch_store_addresses_sql() -> &'static str {
    FETCH_STORE_ADDRESSES_SQL
}
