use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryClient, PageQueryVariables};
use crate::types::common::*;
use crate::types::vault::VaultBalanceChangesListQuery;
use chrono::DateTime;
use reqwest::Url;
use std::cmp::Reverse;

pub struct VaultBalanceChangesListPageQueryClient {
    pub url: Url,
}

impl VaultBalanceChangesListPageQueryClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

impl CynicClient for VaultBalanceChangesListPageQueryClient {
    fn get_base_url(&self) -> Url {
        self.url.clone()
    }
}

impl PageQueryClient<VaultBalanceChangeUnwrapped, PaginationWithIdQueryVariables>
    for VaultBalanceChangesListPageQueryClient
{
    async fn query_page(
        &self,
        variables: PaginationWithIdQueryVariables,
    ) -> Result<Vec<VaultBalanceChangeUnwrapped>, CynicClientError> {
        let res: Result<VaultBalanceChangesListQuery, CynicClientError> = self
            .query::<VaultBalanceChangesListQuery, PaginationWithIdQueryVariables>(variables)
            .await;

        let list: Vec<VaultBalanceChangeUnwrapped> = res?.vault_balance_changes;

        Ok(list)
    }

    /// Sort by timestamp, descending
    fn sort_results(results: Vec<VaultBalanceChangeUnwrapped>) -> Vec<VaultBalanceChangeUnwrapped> {
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|r| {
            Reverse(DateTime::from_timestamp(
                r.timestamp.0.parse::<i64>().unwrap_or(0),
                0,
            ))
        });

        sorted_results
    }
}

impl PageQueryVariables for PaginationWithIdQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
        Self {
            skip,
            first,
            id: self.id.clone(),
        }
    }
}
