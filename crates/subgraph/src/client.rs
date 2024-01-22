use crate::cynic_client::{CynicClient, CynicClientError};
use crate::types::{
    order::{Order, OrderQuery, OrderQueryVariables},
    orders::{Order as OrdersListItem, OrdersQuery as OrdersListQuery},
    vault::{TokenVault, VaultQuery, VaultQueryVariables},
    vaults::{TokenVault as VaultsListItem, VaultsQuery as VaultsListQuery},
};
use cynic::Id;
use reqwest::Url;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderbookSubgraphClientError {
    #[error("Cynic Client Error: {0}")]
    CynicClientError(#[from] CynicClientError),
    #[error("Subgraph query returned no data")]
    Empty,
}

pub struct OrderbookSubgraphClient {
    url: Url,
}

impl CynicClient for OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn orders(&self) -> Result<Vec<OrdersListItem>, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrdersListQuery, ()>(self.url.clone(), ())
            .await?;

        Ok(data.orders)
    }

    pub async fn vaults(&self) -> Result<Vec<VaultsListItem>, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultsListQuery, ()>(self.url.clone(), ())
            .await
            .map_err(|e| OrderbookSubgraphClientError::CynicClientError(e))?;

        Ok(data.token_vaults)
    }

    pub async fn vault(&self, id: Id) -> Result<TokenVault, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultQuery, VaultQueryVariables>(
                self.url.clone(),
                VaultQueryVariables { id: &id },
            )
            .await
            .map_err(|e| OrderbookSubgraphClientError::CynicClientError(e))?;
        let vault = data
            .token_vault
            .ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(vault)
    }

    pub async fn order(&self, id: Id) -> Result<Order, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrderQuery, OrderQueryVariables>(
                self.url.clone(),
                OrderQueryVariables { id: &id },
            )
            .await?;
        let order = data.order.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order)
    }
}
