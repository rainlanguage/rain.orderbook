use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryVariables, PaginationClient, PaginationClientError};
use crate::types::{
    order_detail,
    order_detail::{OrderDetailQuery, OrderDetailQueryVariables},
    orders_list,
    orders_list::{OrdersListQuery, OrdersListQueryVariables},
    vault_balance_change::VaultBalanceChange,
    vault_detail,
    vault_detail::{VaultDetailQuery, VaultDetailQueryVariables},
    vault_list_balance_changes::{
        VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables,
    },
    vaults_list,
    vaults_list::{VaultsListQuery, VaultsListQueryVariables},
};
use crate::PageQueryClient;
use chrono::NaiveDateTime;
use cynic::Id;
use reqwest::Url;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderbookSubgraphClientError {
    #[error(transparent)]
    CynicClientError(#[from] CynicClientError),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error(transparent)]
    PaginationClientError(#[from] PaginationClientError),
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

    pub async fn vault_list_balance_changes(
        &self,
        id: cynic::Id,
        skip: Option<u32>,
        first: Option<u32>,
    ) -> Result<Vec<VaultBalanceChange>, OrderbookSubgraphClientError> {
        let pagination_client = PaginationClient::new(200);
        let res = pagination_client
            .query_paginated(
                skip,
                first,
                VaultListBalanceChangesPageQueryClient {
                    url: self.url.clone(),
                },
                VaultBalanceChangesListQueryVariables {
                    id: &id,
                    skip: Some(0),
                    first: Some(200),
                },
            )
            .await?;

        Ok(res)
    }
}

pub struct VaultListBalanceChangesPageQueryClient {
    url: Url,
}

impl CynicClient for VaultListBalanceChangesPageQueryClient {}

impl<'a> PageQueryClient<VaultBalanceChange, VaultBalanceChangesListQueryVariables<'a>>
    for VaultListBalanceChangesPageQueryClient
{
    async fn query_page(
        &self,
        variables: VaultBalanceChangesListQueryVariables<'a>,
    ) -> Result<Vec<VaultBalanceChange>, CynicClientError> {
        let list = self
            .query::<VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables>(
                self.url.clone(),
                variables,
            )
            .await
            .map(|data| {
                let mut merged: Vec<VaultBalanceChange> = vec![];
                merged.extend(
                    data.vault_deposits
                        .into_iter()
                        .map(VaultBalanceChange::Deposit)
                        .collect::<Vec<VaultBalanceChange>>(),
                );
                merged.extend(
                    data.vault_withdraws
                        .into_iter()
                        .map(VaultBalanceChange::Withdraw)
                        .collect::<Vec<VaultBalanceChange>>(),
                );

                merged
            })?;

        Ok(list)
    }

    fn sort_results(results: Vec<VaultBalanceChange>) -> Vec<VaultBalanceChange> {
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|r| {
            let timestamp = match r {
                VaultBalanceChange::Deposit(v) => v.timestamp.clone().0,
                VaultBalanceChange::Withdraw(v) => v.timestamp.clone().0,
            };

            NaiveDateTime::from_timestamp_opt(timestamp.parse::<i64>().unwrap_or(0), 0)
        });

        sorted_results
    }
}

impl<'a> PageQueryVariables for VaultBalanceChangesListQueryVariables<'a> {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
        Self {
            skip,
            first,
            id: self.id,
        }
    }
}
