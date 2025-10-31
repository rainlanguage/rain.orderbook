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

    #[test]
    fn fetch_stmt_includes_where_id_1() {
        let stmt = fetch_db_metadata_stmt();
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("where id = 1"));
        assert!(lower.contains("select id, db_schema_version, created_at, updated_at"));
    }

    #[test]
    fn db_metadata_row_serde_roundtrip_none_fields() {
        let row = DbMetadataRow {
            id: 1,
            db_schema_version: 7,
            created_at: None,
            updated_at: None,
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: DbMetadataRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }

    #[test]
    fn db_metadata_row_serde_roundtrip_some_fields() {
        let row = DbMetadataRow {
            id: 1,
            db_schema_version: 42,
            created_at: Some("2024-01-01T00:00:00Z".to_owned()),
            updated_at: Some("2024-01-02T00:00:00Z".to_owned()),
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: DbMetadataRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }
}
