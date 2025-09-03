use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersFilter {
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOrdersResponse {
    #[serde(alias = "orderHash")]
    pub order_hash: String,
    pub owner: String,
    #[serde(alias = "blockTimestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "blockNumber")]
    pub block_number: u64,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    #[serde(alias = "tradeCount")]
    pub trade_count: u64,
    pub status: String,
}

impl LocalDbQuery {
    pub async fn fetch_orders(
        db_callback: &js_sys::Function,
        filter: FetchOrdersFilter,
    ) -> Result<Vec<FetchOrdersResponse>, LocalDbQueryError> {
        let filter_str = match filter {
            FetchOrdersFilter::All => "all",
            FetchOrdersFilter::Active => "active",
            FetchOrdersFilter::Inactive => "inactive",
        };

        let sql = QUERY.replace("'?filter'", &format!("'{}'", filter_str));

        LocalDbQuery::execute_query_json::<Vec<FetchOrdersResponse>>(db_callback, &sql).await
    }
}
