use crate::local_db::query::SqlStatement;

use serde::{Deserialize, Serialize};

pub const FETCH_DB_METADATA_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbMetadataRow {
    pub id: u32,
    pub db_schema_version: u32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub fn fetch_db_metadata_stmt() -> SqlStatement {
    SqlStatement::new(FETCH_DB_METADATA_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_stmt_is_static_and_param_free() {
        let stmt = fetch_db_metadata_stmt();
        assert_eq!(stmt.sql(), FETCH_DB_METADATA_SQL);
        assert!(stmt.params().is_empty());
        assert!(stmt.sql().to_lowercase().contains("from db_metadata"));
    }
}
