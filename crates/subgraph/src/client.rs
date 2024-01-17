use crate::types::orders::{Order, OrdersQuery, OrdersQueryVariables, OrdersQueryWhere};
use anyhow::Result;
use cynic::{GraphQlResponse, QueryBuilder};
use once_cell::sync::Lazy;
use reqwest::Url;

static BASE_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://api.thegraph.com/subgraphs/name/siddharth2207/rainorderbook").unwrap()
});

pub struct OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub async fn orders(active: Option<bool>) -> Result<Vec<Order>> {
        let request_body = OrdersQuery::build(OrdersQueryVariables {
            where_filter: Some(OrdersQueryWhere { active }),
        });

        let response = reqwest::Client::new()
            .post((*BASE_URL).clone())
            .json(&request_body)
            .send()
            .await?;

        let orders_response: GraphQlResponse<OrdersQuery> =
            response.json::<GraphQlResponse<OrdersQuery>>().await?;

        let orders = if let Some(data) = orders_response.data {
            data.orders
        } else {
            vec![]
        };

        Ok(orders)
    }
}
