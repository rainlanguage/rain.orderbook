use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResponse {
    pub name: String,
}

impl LocalDbQuery {
    pub async fn fetch_all_tables(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<TableResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_with_callback::<Vec<TableResponse>>(db_callback, QUERY).await
    }
}
