use super::*;

impl OrderbookSubgraphClient {
    /// Fetch single vault
    pub async fn vault_detail(&self, id: Id) -> Result<SgVault, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgVaultDetailQuery, SgIdQueryVariables>(SgIdQueryVariables { id: &id })
            .await?;
        let vault = data.vault.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(vault)
    }

    /// Fetch all vaults, paginated
    pub async fn vaults_list(
        &self,
        filter_args: SgVaultsListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgVault>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);

        let mut filters = SgVaultsListQueryFilters {
            owner_in: filter_args.owners.clone(),
            balance_gt: None,
        };

        if filter_args.hide_zero_balance {
            filters.balance_gt = Some(SgBigInt("0".to_string()));
        }

        let variables = SgVaultsListQueryVariables {
            first: pagination_variables.first,
            skip: pagination_variables.skip,
            filters: if !filter_args.owners.is_empty() || filter_args.hide_zero_balance {
                Some(filters)
            } else {
                None
            },
        };

        let data = self
            .query::<SgVaultsListQuery, SgVaultsListQueryVariables>(variables)
            .await?;

        Ok(data.vaults)
    }

    /// Fetch all pages of vaults_list query
    pub async fn vaults_list_all(&self) -> Result<Vec<SgVault>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .vaults_list(
                    SgVaultsListFilterArgs {
                        owners: vec![],
                        hide_zero_balance: true,
                    },
                    SgPaginationArgs {
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

    /// Fetch all vault deposits + withdrawals merged paginated, for a single vault
    pub async fn vault_balance_changes_list(
        &self,
        id: cynic::Id,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgVaultBalanceChangeUnwrapped>, OrderbookSubgraphClientError> {
        let pagination_vars = Self::parse_pagination_args(pagination_args);
        let res = self
            .query_paginated(
                pagination_vars,
                VaultBalanceChangesListPageQueryClient::new(self.url.clone()),
                SgPaginationWithIdQueryVariables {
                    id: SgBytes(id.inner().to_string()),
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
    ) -> Result<Vec<SgVaultBalanceChangeUnwrapped>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .vault_balance_changes_list(
                    id.clone(),
                    SgPaginationArgs {
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
