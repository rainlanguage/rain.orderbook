use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PaginationArgs, PaginationClient, PaginationClientError};
use crate::types::{
    order_detail,
    order_detail::{OrderDetailQuery, OrderDetailQueryVariables},
    order_take_detail,
    order_take_detail::{OrderTakeDetailQuery, OrderTakeDetailQueryVariables},
    order_takes_list,
    order_takes_list::{OrderTakesListQuery, OrderTakesListQueryVariables},
    orders_list,
    orders_list::{OrdersListQuery, OrdersListQueryVariables},
    vault_balance_change::VaultBalanceChange,
    vault_balance_changes_list::VaultBalanceChangesListQueryVariables,
    vault_detail,
    vault_detail::{VaultDetailQuery, VaultDetailQueryVariables},
    vaults_list,
    vaults_list::{VaultsListQuery, VaultsListQueryVariables},
};
use crate::vault_balance_changes_query::VaultBalanceChangesListPageQueryClient;

use cynic::Id;
use reqwest::Url;
use thiserror::Error;

const ALL_PAGES_QUERY_PAGE_SIZE: u16 = 200;

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

impl CynicClient for OrderbookSubgraphClient {
    fn get_base_url(&self) -> Url {
        self.url.clone()
    }
}
impl PaginationClient for OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    /// Fetch single order
    pub async fn order_detail(
        &self,
        id: Id,
    ) -> Result<order_detail::Order, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrderDetailQuery, OrderDetailQueryVariables>(OrderDetailQueryVariables {
                id: &id,
            })
            .await?;
        let order = data.order.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order)
    }

    /// Fetch all orders, paginated
    pub async fn orders_list(
        &self,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<orders_list::Order>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<OrdersListQuery, OrdersListQueryVariables>(OrdersListQueryVariables {
                first: pagination_variables.first,
                skip: pagination_variables.skip,
            })
            .await?;

        Ok(data.orders)
    }

    /// Fetch all pages of orders_list query
    pub async fn orders_list_all(
        &self,
    ) -> Result<Vec<orders_list::Order>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .orders_list(PaginationArgs {
                    page,
                    page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                })
                .await?;
            if page_data.is_empty() {
                break;
            } else {
                all_pages_merged.extend(page_data);
                page += 1
            }
        }
        Ok(all_pages_merged)
    }

    /// Fetch single order take
    pub async fn order_take_detail(
        &self,
        id: Id,
    ) -> Result<order_take_detail::TakeOrderEntity, OrderbookSubgraphClientError> {
        let data = self
            .query::<OrderTakeDetailQuery, OrderTakeDetailQueryVariables>(
                OrderTakeDetailQueryVariables { id: &id },
            )
            .await?;
        let order_take = data
            .take_order_entity
            .ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order_take)
    }

    /// Fetch all order takes paginated for a single order
    pub async fn order_takes_list(
        &self,
        order_id: cynic::Id,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<order_takes_list::TakeOrderEntity>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<OrderTakesListQuery, OrderTakesListQueryVariables>(
                OrderTakesListQueryVariables {
                    id: &order_id,
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                },
            )
            .await?;

        Ok(data.take_order_entities)
    }

    /// Fetch all pages of order_takes_list query
    pub async fn order_takes_list_all(
        &self,
        order_id: cynic::Id,
    ) -> Result<Vec<order_takes_list::TakeOrderEntity>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .order_takes_list(
                    order_id.clone(),
                    PaginationArgs {
                        page,
                        page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                    },
                )
                .await?;
            if page_data.is_empty() {
                break;
            } else {
                all_pages_merged.extend(page_data);
                page += 1
            }
        }
        Ok(all_pages_merged)
    }

    /// Fetch single vault
    pub async fn vault_detail(
        &self,
        id: Id,
    ) -> Result<vault_detail::TokenVault, OrderbookSubgraphClientError> {
        let data = self
            .query::<VaultDetailQuery, VaultDetailQueryVariables>(VaultDetailQueryVariables {
                id: &id,
            })
            .await?;
        let vault = data
            .token_vault
            .ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(vault)
    }

    /// Fetch all vaults, paginated
    pub async fn vaults_list(
        &self,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<vaults_list::TokenVault>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<VaultsListQuery, VaultsListQueryVariables>(VaultsListQueryVariables {
                first: pagination_variables.first,
                skip: pagination_variables.skip,
            })
            .await?;

        Ok(data.token_vaults)
    }

    /// Fetch all pages of vaults_list query
    pub async fn vaults_list_all(
        &self,
    ) -> Result<Vec<vaults_list::TokenVault>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .vaults_list(PaginationArgs {
                    page,
                    page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                })
                .await?;
            if page_data.is_empty() {
                break;
            } else {
                all_pages_merged.extend(page_data);
                page += 1
            }
        }
        Ok(all_pages_merged)
    }

    /// Fetch all vault deposits + withdrawals merged paginated, for a single vault
    pub async fn vault_balance_changes_list(
        &self,
        id: cynic::Id,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<VaultBalanceChange>, OrderbookSubgraphClientError> {
        let pagination_vars = Self::parse_pagination_args(pagination_args);
        let res = self
            .query_paginated(
                pagination_vars,
                VaultBalanceChangesListPageQueryClient::new(self.url.clone()),
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

    /// Fetch all pages of vault_balance_changes_list query
    pub async fn vault_balance_changes_list_all(
        &self,
        id: cynic::Id,
    ) -> Result<Vec<VaultBalanceChange>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .vault_balance_changes_list(
                    id.clone(),
                    PaginationArgs {
                        page,
                        page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                    },
                )
                .await?;
            if page_data.is_empty() {
                break;
            } else {
                all_pages_merged.extend(page_data);
                page += 1
            }
        }
        Ok(all_pages_merged)
    }
}
