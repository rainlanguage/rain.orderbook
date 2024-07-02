use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryClient, PageQueryVariables};
use crate::types::vault_balance_changes_list::VaultBalanceChange;
use crate::types::vault_balance_changes_list::{
    VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables,
};
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

impl PageQueryClient<VaultBalanceChange, VaultBalanceChangesListQueryVariables>
    for VaultBalanceChangesListPageQueryClient
{
    async fn query_page(
        &self,
        variables: VaultBalanceChangesListQueryVariables,
    ) -> Result<Vec<VaultBalanceChange>, CynicClientError> {
        let res: Result<VaultBalanceChangesListQuery, CynicClientError> = self
            .query::<VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables>(variables)
            .await;

        let list: Vec<VaultBalanceChange> = res?.vault_balance_changes;

        Ok(list)
    }

    /// Sort by timestamp, descending
    fn sort_results(results: Vec<VaultBalanceChange>) -> Vec<VaultBalanceChange> {
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

impl<'a> PageQueryVariables for VaultBalanceChangesListQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
        Self {
            skip,
            first,
            id: self.id.clone(),
        }
    }
}
