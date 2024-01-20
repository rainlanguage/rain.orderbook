use crate::cynic_client::CynicClient;
use crate::types::{
    order::{Order, OrderQuery, OrderQueryVariables},
    orders::{Order as OrdersListItem, OrdersQuery as OrdersListQuery},
    vault::{Vault, VaultQuery, VaultQueryVariables},
    vaults::{TokenVault as VaultsListItem, VaultsQuery as VaultsListQuery},
};
use anyhow::{anyhow, Result};
use cynic::Id;
use reqwest::Url;

pub struct OrderbookSubgraphClient {
    url: Url,
}

impl CynicClient for OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn orders(&self) -> Result<Vec<OrdersListItem>> {
        let data = self
            .query::<OrdersListQuery, ()>(self.url.clone(), ())
            .await?;

        Ok(data.orders)
    }

    pub async fn vaults(&self) -> Result<Vec<VaultsListItem>> {
        let data = self
            .query::<VaultsListQuery, ()>(self.url.clone(), ())
            .await?;

        Ok(data.token_vaults)
    }

    pub async fn vault(&self, id: Id) -> Result<Vault> {
        let data = self
            .query::<VaultQuery, VaultQueryVariables>(
                self.url.clone(),
                VaultQueryVariables { id: &id },
            )
            .await?;
        let vault = data.vault.ok_or(anyhow!("Vault not found"))?;

        Ok(vault)
    }

    pub async fn order(&self, id: Id) -> Result<Order> {
        let data = self
            .query::<OrderQuery, OrderQueryVariables>(
                self.url.clone(),
                OrderQueryVariables { id: &id },
            )
            .await?;
        let order = data.order.ok_or(anyhow!("Order not found"))?;

        Ok(order)
    }
}
