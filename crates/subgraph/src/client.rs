use crate::cynic_client::CynicClient;
use crate::types::{
    orders::{Order, OrdersQuery},
    vault::{Vault, VaultQuery, VaultQueryVariables},
    vaults::{Vault as VaultsListItem, VaultsQuery as VaultsListQuery},
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

    pub async fn orders(&self) -> Result<Vec<Order>> {
        let data = self.query::<OrdersQuery, ()>(self.url.clone(), ()).await?;

        Ok(data.orders)
    }

    pub async fn vaults(&self) -> Result<Vec<VaultsListItem>> {
        let data = self
            .query::<VaultsListQuery, ()>(self.url.clone(), ())
            .await?;

        Ok(data.vaults)
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
}
