use crate::types::{
    orders::{Order, OrdersQuery},
    vault::{Vault, VaultQuery, VaultQueryVariables},
    vaults::{TokenVault, VaultsQuery},
};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, Id, QueryBuilder};
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

    pub async fn vault(&self, id: Id) -> Result<Vault> {
        let request_body = VaultQuery::build(VaultQueryVariables { id: &id });

        let response = reqwest::Client::new()
            .post(self.url.clone())
            .json(&request_body)
            .send()
            .await?;

        let vault_response: GraphQlResponse<VaultQuery> =
            response.json::<GraphQlResponse<VaultQuery>>().await?;

        let vault = vault_response
            .data
            .ok_or(anyhow!("Graphql Errors: {:?}", vault_response.errors))?
            .vault
            .ok_or(anyhow!("Vault not found"))?;

        Ok(vault)
    }
}
