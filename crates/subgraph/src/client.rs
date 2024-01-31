use crate::cynic_client::{CynicClient, CynicClientError};
use crate::types::{
    order_detail,
    order_detail::{OrderDetailQuery, OrderDetailQueryVariables},
    orders_list,
    orders_list::{OrdersListQuery, OrdersListQueryVariables},
    vault_detail,
    vault_detail::{VaultDetailQuery, VaultDetailQueryVariables},
    vaults_list,
    vaults_list::{VaultsListQuery, VaultsListQueryVariables},
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

    pub async fn orders_list<T: Into<OrdersListQueryVariables>>(
        &self,
        variables: T,
    ) -> Result<Vec<orders_list::Order>, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrdersListQuery, OrdersListQueryVariables>(self.url.clone(), variables.into())
            .await?;

        Ok(data.orders)
    }

    pub async fn vaults_list<T: Into<VaultsListQueryVariables>>(
        &self,
        variables: T,
    ) -> Result<Vec<vaults_list::TokenVault>, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultsListQuery, VaultsListQueryVariables>(self.url.clone(), variables.into())
            .await?;

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
            .await?;
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
