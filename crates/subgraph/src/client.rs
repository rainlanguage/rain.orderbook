use crate::cynic_client::{CynicClient, CynicClientError};
use crate::types::{
    order_detail,
    order_detail::{OrderDetailQuery, OrderDetailQueryVariables},
    orders_list,
    orders_list::{OrdersListQuery, OrdersListQueryVariables},
    vault_balancechange::VaultBalanceChange,
    vault_balancechanges_list::{
        VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables,
    },
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

    /// Helper to merge and paginate two graphql query response lists
    /// 1. Fetch a page
    /// 2. Insert page contents in merged vec & sort by timetamp
    /// 3. Check if skip & first variables have been met
    ///      - If not, repeat 2-4
    ///      - If so, return merged vec
    pub async fn vault_balancechanges_list(
        &self,
        id: cynic::Id,
        skip: Option<i32>,
        first: Option<i32>,
    ) -> Result<Vec<VaultBalanceChange>, OrderbookSubgraphClientError> {
        let target_len = if let Some(first_data) = first {
            Some(skip.unwrap_or(0) + first_data)
        } else {
            None
        };

        let mut results = vec![];
        let mut more_results_available = true;
        let mut page_skip = 0;
        let page_first = 200;

        // Fetch subgraph pages until out of results, or received results meet desired length:
        while (target_len.is_none() || results.len() < target_len.unwrap() as usize)
            && more_results_available
        {
            // Fetch a page
            let res = self
                .query::<VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables>(
                    self.url.clone(),
                    VaultBalanceChangesListQueryVariables {
                        id: &id,
                        skip: Some(page_skip),
                        first: Some(page_first),
                    },
                )
                .await;

            let _ = match res {
                Ok(data) => {
                    // No results
                    if data.vault_deposits.len() == 0 && data.vault_withdraws.len() == 0 {
                        more_results_available = false;
                        Ok(())
                    }
                    // Results received, append to merged vec and re-sort
                    else {
                        results.extend(
                            data.vault_deposits
                                .into_iter()
                                .map(VaultBalanceChange::Deposit)
                                .collect::<Vec<VaultBalanceChange>>(),
                        );
                        results.extend(
                            data.vault_withdraws
                                .into_iter()
                                .map(VaultBalanceChange::Withdraw)
                                .collect::<Vec<VaultBalanceChange>>(),
                        );
                        results.sort_by_key(|b| match b {
                            VaultBalanceChange::Deposit(d) => d.timestamp.0.clone(),
                            VaultBalanceChange::Withdraw(w) => w.timestamp.0.clone(),
                        });

                        page_skip += page_first;
                        Ok(())
                    }
                }
                // No results
                Err(CynicClientError::Empty) => {
                    more_results_available = false;
                    Ok(())
                }
                Err(e) => Err(OrderbookSubgraphClientError::CynicClientError(e)),
            }?;
        }

        // Slice the desired page from of the merged results
        let mut results_page = results.clone();
        if let Some(s) = skip {
            if results_page.len() > s as usize {
                results_page = results_page[s as usize..].to_vec();
            } else {
                results_page = vec![];
            }
        }
        if let Some(f) = first {
            if results_page.len() > f as usize {
                results_page = results_page[..f as usize].to_vec();
            }
        }

        Ok(results_page)
    }
}
