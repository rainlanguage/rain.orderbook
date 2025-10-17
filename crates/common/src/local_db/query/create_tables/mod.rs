pub const CREATE_TABLES_SQL: &str = include_str!("query.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "sync_status",
    "raw_events",
    "deposits",
    "withdrawals",
    "order_events",
    "order_ios",
    "take_orders",
    "take_order_contexts",
    "context_values",
    "clear_v3_events",
    "after_clear_v2_events",
    "meta_events",
    "erc20_tokens",
    "interpreter_store_sets",
];

pub fn create_tables_sql() -> &'static str {
    CREATE_TABLES_SQL
}
