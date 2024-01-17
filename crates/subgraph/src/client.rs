use crate::cynic_client::CynicClient;
use crate::types::{
    orders::{Order, OrdersQuery},
    vaults::{TokenVault, VaultsQuery},
};
use anyhow::Result;
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

    pub async fn vaults(&self) -> Result<Vec<TokenVault>> {
        let data = self.query::<VaultsQuery, ()>(self.url.clone(), ()).await?;

        Ok(data.token_vaults)
    }
}
