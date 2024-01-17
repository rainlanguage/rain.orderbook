use crate::types::{
    orders::{Order, OrdersQuery},
    vaults::{TokenVault, VaultsQuery},
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

        match orders_response.errors {
            Some(errors) => anyhow!(
                "Graphql: {}",
                errors.iter().map(|e| e.message).collect().join(", ")
            ),
            None => {
                orders_response
                    .data
                    .ok_or(anyhow!("Subgraph query returned no data"))
                    .orders
            }
        }
    }

    pub async fn vaults(&self) -> Result<Vec<TokenVault>> {
        let request_body = VaultsQuery::build(());

        let response = reqwest::Client::new()
            .post(self.url.clone())
            .json(&request_body)
            .send()
            .await?;

        let vaults_response: GraphQlResponse<VaultsQuery> =
            response.json::<GraphQlResponse<VaultsQuery>>().await?;

        let vaults = if let Some(data) = vaults_response.data {
            data.token_vaults
        } else {
            vec![]
        };

        Ok(vaults)
    }
}
