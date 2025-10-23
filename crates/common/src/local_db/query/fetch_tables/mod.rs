use serde::{Deserialize, Serialize};

pub const FETCH_TABLES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableResponse {
    pub name: String,
}

use crate::local_db::query::SqlStatement;

pub fn fetch_tables_stmt() -> SqlStatement {
    SqlStatement::new(FETCH_TABLES_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_uses_static_sql_and_no_params() {
        let stmt = fetch_tables_stmt();
        assert_eq!(stmt.sql, FETCH_TABLES_SQL);
        assert!(stmt.params.is_empty());
        assert!(stmt.sql.to_lowercase().contains("from sqlite_master"));
    }
}
