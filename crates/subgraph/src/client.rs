use crate::cynic_client::{CynicClient, CynicClientError};
use crate::types::{
    order_detail,
    order_detail::{OrderDetailQuery, OrderDetailQueryVariables},
    orders_list,
    orders_list::OrdersListQuery,
    vault_detail,
    vault_detail::{VaultDetailQuery, VaultDetailQueryVariables},
    vaults_list,
    vaults_list::VaultsListQuery,
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

    pub async fn orders_list(
        &self,
    ) -> Result<Vec<orders_list::Order>, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrdersListQuery, ()>(self.url.clone(), ())
            .await?;

        Ok(data.orders)
    }

    pub async fn vaults_list(
        &self,
    ) -> Result<Vec<vaults_list::TokenVault>, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultsListQuery, ()>(self.url.clone(), ())
            .await
            .map_err(OrderbookSubgraphClientError::CynicClientError)?;

        Ok(data.token_vaults)
    }

    pub async fn vault_detail(
        &self,
        id: Id,
    ) -> Result<vault_detail::TokenVault, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultDetailQuery, VaultDetailQueryVariables>(
                self.url.clone(),
                VaultDetailQueryVariables { id: &id },
            )
            .await
            .map_err(OrderbookSubgraphClientError::CynicClientError)?;
        let vault = data
            .token_vault
            .ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(vault)
    }

    pub async fn order_detail(
        &self,
        id: Id,
    ) -> Result<order_detail::Order, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrderDetailQuery, OrderDetailQueryVariables>(
                self.url.clone(),
                OrderDetailQueryVariables { id: &id },
            )
            .await?;
        let order = data.order.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order)
    }
}
