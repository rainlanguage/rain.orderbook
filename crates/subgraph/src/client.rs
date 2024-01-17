use crate::types::{
    orders::{Order, OrdersQuery},
    vaults::{Vault, VaultsQuery},
};
use anyhow::Result;
use cynic::{GraphQlResponse, QueryBuilder};
use reqwest::Url;

pub struct OrderbookSubgraphClient {
    url: Url,
}

impl OrderbookSubgraphClient {
    pub async fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn orders(&self) -> Result<Vec<Order>> {
        let request_body = OrdersQuery::build(());

        let response = reqwest::Client::new()
            .post(self.url.clone())
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

    pub async fn vaults() -> Result<Vec<Vault>> {
        let request_body = VaultsQuery::build(());

        let response = reqwest::Client::new()
            .post(self.url.clone())
            .json(&request_body)
            .send()
            .await?;

        let vaults_response: GraphQlResponse<VaultsQuery> =
            response.json::<GraphQlResponse<VaultsQuery>>().await?;

        let vaults = if let Some(data) = vaults_response.data {
            data.vaults
        } else {
            vec![]
        };

        Ok(vaults)
    }
}
