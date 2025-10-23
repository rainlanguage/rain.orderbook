use serde::{Deserialize, Serialize};

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: String,
}

use crate::local_db::query::SqlStatement;

pub fn fetch_store_addresses_stmt() -> SqlStatement {
    SqlStatement::new(FETCH_STORE_ADDRESSES_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_is_static_and_param_free() {
        let stmt = fetch_store_addresses_stmt();
        assert_eq!(stmt.sql, FETCH_STORE_ADDRESSES_SQL);
        assert!(stmt.params.is_empty());
        assert!(stmt
            .sql
            .to_lowercase()
            .contains("select distinct lower(store_address)"));
    }
}
