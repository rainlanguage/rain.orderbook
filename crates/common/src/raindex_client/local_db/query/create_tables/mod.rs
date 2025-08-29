use super::*;

const QUERY: &str = include_str!("query.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "sync_status",
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
];

impl LocalDbQuery {
    pub async fn create_tables(db_callback: &js_sys::Function) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_with_callback::<()>(db_callback, QUERY).await
    }
}
