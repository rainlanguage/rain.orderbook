use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryClient, PageQueryVariables};
use crate::types::common::*;
use crate::types::vault::SgVaultBalanceChangesListQuery;
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
    fn get_base_url(&self) -> &Url {
        &self.url
    }
}

impl PageQueryClient<SgVaultBalanceChangeUnwrapped, SgPaginationWithIdQueryVariables>
    for VaultBalanceChangesListPageQueryClient
{
    async fn query_page(
        &self,
        variables: SgPaginationWithIdQueryVariables,
    ) -> Result<Vec<SgVaultBalanceChangeUnwrapped>, CynicClientError> {
        let res: Result<SgVaultBalanceChangesListQuery, CynicClientError> = self
            .query::<SgVaultBalanceChangesListQuery, SgPaginationWithIdQueryVariables>(variables)
            .await;

        let list: Vec<SgVaultBalanceChangeUnwrapped> = res?.vault_balance_changes;

        Ok(list)
    }

    /// Sort by timestamp, descending
    fn sort_results(
        results: Vec<SgVaultBalanceChangeUnwrapped>,
    ) -> Vec<SgVaultBalanceChangeUnwrapped> {
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

impl PageQueryVariables for SgPaginationWithIdQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
        Self {
            skip,
            first,
            id: self.id.clone(),
        }
    }
}
