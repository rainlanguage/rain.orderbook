use super::*;

const QUERY: &str = include_str!("query.sql");

impl LocalDbQuery {
    pub async fn update_last_synced_block(
        db_callback: &js_sys::Function,
        block_number: u64,
    ) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_with_callback::<()>(
            db_callback,
            &QUERY.replace("?block_number", &block_number.to_string()),
        )
        .await
    }
}
