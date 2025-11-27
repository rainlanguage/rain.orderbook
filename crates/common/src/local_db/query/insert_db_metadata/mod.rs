use crate::local_db::query::{SqlStatement, SqlValue};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn insert_db_metadata_stmt(version: u32) -> SqlStatement {
    SqlStatement::new_with_params(QUERY_TEMPLATE, [SqlValue::from(version)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_stmt_binds_version() {
        let stmt = insert_db_metadata_stmt(7);
        assert!(stmt
            .sql()
            .to_lowercase()
            .starts_with("insert into db_metadata"));
        assert_eq!(stmt.params().len(), 1);
    }

    #[test]
    fn insert_stmt_sql_matches_template_and_values_clause() {
        let stmt = insert_db_metadata_stmt(1);
        assert_eq!(stmt.sql(), QUERY_TEMPLATE);
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("insert into db_metadata (id, db_schema_version) values (1, ?1)"));
    }

    #[test]
    fn insert_stmt_param_value_and_type() {
        let version = 123u32;
        let stmt = insert_db_metadata_stmt(version);
        let params = stmt.params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], SqlValue::U64(version as u64));
    }
}
