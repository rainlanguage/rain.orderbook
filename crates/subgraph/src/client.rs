use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{
    PageQueryVariables, PaginationArgs, PaginationClient, PaginationClientError,
};
use crate::types::{
    order_clears_list,
    order_clears_list::{OrderClearsListQuery, OrderClearsListQueryVariables},
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
    order_takes_list,
    order_takes_list::{OrderTakesListQuery, OrderTakesListQueryVariables}
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
impl PaginationClient for OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn orders_list(
        &self,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<orders_list::Order>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<OrdersListQuery, OrdersListQueryVariables>(
                self.url.clone(),
                OrdersListQueryVariables {
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                },
            )
            .await?;

        Ok(data.orders)
    }

    pub async fn vaults_list(
        &self,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<vaults_list::TokenVault>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<VaultsListQuery, VaultsListQueryVariables>(
                self.url.clone(),
                VaultsListQueryVariables {
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                },
            )
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
        pagination_args: PaginationArgs,
    ) -> Result<Vec<VaultBalanceChange>, OrderbookSubgraphClientError> {
        let pagination_vars = Self::parse_pagination_args(pagination_args);
        let res = self
            .query_paginated(
                pagination_vars,
                VaultListBalanceChangesPageQueryClient {
                    url: self.url.clone(),
                },
                VaultBalanceChangesListQueryVariables {
                    id: &id,
                    skip: Some(0),
                    first: Some(200),
                },
                200,
            )
            .await?;

        Ok(res)
    }

    pub async fn order_clears_list(
        &self,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<order_clears_list::OrderClear>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<OrderClearsListQuery, OrderClearsListQueryVariables>(
                self.url.clone(),
                OrderClearsListQueryVariables {
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                },
            )
            .await?;

        Ok(data.order_clears)
    }

    pub async fn order_takes_list(
        &self,
        order_id: cynic::Id,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<order_takes_list::TakeOrderEntity>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<OrderTakesListQuery, OrderTakesListQueryVariables>(
                self.url.clone(),
                OrderTakesListQueryVariables {
                    id: &order_id,
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                },
            )
            .await?;

        Ok(data.take_order_entities)
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
