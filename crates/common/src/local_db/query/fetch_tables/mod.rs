use serde::{Deserialize, Serialize};

pub const FETCH_TABLES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableResponse {
    pub name: String,
}

pub fn fetch_tables_sql() -> &'static str {
    FETCH_TABLES_SQL
}
