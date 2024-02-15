use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryClient, PageQueryVariables};
use crate::types::vault_balance_change::VaultBalanceChange;
use crate::types::vault_balance_changes_list::{
    VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables,
};
use chrono::NaiveDateTime;
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

impl<'a> PageQueryClient<VaultBalanceChange, VaultBalanceChangesListQueryVariables<'a>>
    for VaultBalanceChangesListPageQueryClient
{
    async fn query_page(
        &self,
        variables: VaultBalanceChangesListQueryVariables<'a>,
    ) -> Result<Vec<VaultBalanceChange>, CynicClientError> {
        let list = self
            .query::<VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables>(variables)
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

    /// Sort by timestamp, descending
    fn sort_results(results: Vec<VaultBalanceChange>) -> Vec<VaultBalanceChange> {
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|r| {
            let timestamp = match r {
                VaultBalanceChange::Deposit(v) => v.timestamp.clone().0,
                VaultBalanceChange::Withdraw(v) => v.timestamp.clone().0,
            };

            Reverse(NaiveDateTime::from_timestamp_opt(timestamp.parse::<i64>().unwrap_or(0), 0))
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
