pub const CLEAR_TABLES_SQL: &str = include_str!("query.sql");

/// Returns the SQL snippet that drops all local database tables and performs
/// cleanup (transaction + vacuum). Backends should execute this text verbatim.
pub fn clear_tables_sql() -> &'static str {
    CLEAR_TABLES_SQL
}
