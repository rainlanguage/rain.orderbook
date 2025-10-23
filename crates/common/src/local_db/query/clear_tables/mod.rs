use crate::local_db::query::SqlStatement;

pub const CLEAR_TABLES_SQL: &str = include_str!("query.sql");

/// Returns the SQL statement that drops all local database tables and performs
/// cleanup (transaction + vacuum). No parameters are bound for this script.
pub fn clear_tables_stmt() -> SqlStatement {
    SqlStatement::new(CLEAR_TABLES_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_is_static_and_param_free() {
        let stmt = clear_tables_stmt();
        assert_eq!(stmt.sql, CLEAR_TABLES_SQL);
        assert!(stmt.params.is_empty());
        let lower = stmt.sql.to_lowercase();
        assert!(lower.contains("begin transaction"));
        assert!(lower.contains("drop table if exists sync_status"));
        assert!(lower.contains("vacuum"));
    }
}
