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
            }
            all_pages_merged.extend(page_data);
            page += 1
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
            }
            all_pages_merged.extend(page_data);
            page += 1
        }
        Ok(all_pages_merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgTransaction, SgVault,
        SgVaultBalanceChangeUnwrapped, SgVaultBalanceChangeVault, SgVaultsListFilterArgs,
    };
    use cynic::Id;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    fn setup_client(server: &MockServer) -> OrderbookSubgraphClient {
        let url = Url::parse(&server.url("")).unwrap();
        OrderbookSubgraphClient::new(url)
    }

    fn default_sg_erc20() -> SgErc20 {
        SgErc20 {
            id: SgBytes("0xTokenId".to_string()),
            address: SgBytes("0xTokenAddress".to_string()),
            name: Some("Test Token".to_string()),
            symbol: Some("TTK".to_string()),
            decimals: Some(SgBigInt("18".to_string())),
        }
    }

    fn default_sg_orderbook() -> SgOrderbook {
        SgOrderbook {
            id: SgBytes("0xOrderbookId".to_string()),
        }
    }

    fn default_sg_order_as_io() -> SgOrderAsIO {
        SgOrderAsIO {
            id: SgBytes("0xOrderId".to_string()),
            order_hash: SgBytes("0xOrderHash".to_string()),
            active: true,
        }
    }

    fn default_sg_vault() -> SgVault {
        SgVault {
            id: SgBytes("0xVaultIdDefault".to_string()),
            owner: SgBytes("0xOwnerAddressDefault".to_string()),
            vault_id: SgBigInt("1234567890".to_string()),
            balance: SgBigInt("1000000000000000000".to_string()),
            token: default_sg_erc20(),
            orderbook: default_sg_orderbook(),
            orders_as_output: vec![default_sg_order_as_io()],
            orders_as_input: vec![],
            balance_changes: vec![],
        }
    }

    fn assert_sg_vault_eq(actual: &SgVault, expected: &SgVault) {
        assert_eq!(actual.id, expected.id, "Vault ID mismatch");
        assert_eq!(actual.owner, expected.owner, "Vault owner mismatch");
        assert_eq!(
            actual.vault_id, expected.vault_id,
            "Vault vault_id mismatch"
        );
        assert_eq!(actual.balance, expected.balance, "Vault balance mismatch");
        assert_eq!(actual.token.id, expected.token.id, "Token ID mismatch");
        assert_eq!(
            actual.token.address, expected.token.address,
            "Token address mismatch"
        );
        assert_eq!(
            actual.token.name, expected.token.name,
            "Token name mismatch"
        );
        assert_eq!(
            actual.token.symbol, expected.token.symbol,
            "Token symbol mismatch"
        );
        assert_eq!(
            actual.token.decimals, expected.token.decimals,
            "Token decimals mismatch"
        );
        assert_eq!(
            actual.orderbook.id, expected.orderbook.id,
            "Orderbook ID mismatch"
        );
        assert_eq!(
            actual.orders_as_output.len(),
            expected.orders_as_output.len(),
            "Orders as output length mismatch"
        );
        for (act_o, exp_o) in actual
            .orders_as_output
            .iter()
            .zip(expected.orders_as_output.iter())
        {
            assert_eq!(act_o.id, exp_o.id);
            assert_eq!(act_o.order_hash, exp_o.order_hash);
            assert_eq!(act_o.active, exp_o.active);
        }
        assert_eq!(
            actual.orders_as_input.len(),
            expected.orders_as_input.len(),
            "Orders as input length mismatch"
        );
        for (act_i, exp_i) in actual
            .orders_as_input
            .iter()
            .zip(expected.orders_as_input.iter())
        {
            assert_eq!(act_i.id, exp_i.id);
            assert_eq!(act_i.order_hash, exp_i.order_hash);
            assert_eq!(act_i.active, exp_i.active);
        }
    }

    fn default_sg_transaction() -> SgTransaction {
        SgTransaction {
            id: SgBytes("0xTransactionId".to_string()),
            from: SgBytes("0xSenderAddress".to_string()),
            block_number: SgBigInt("100".to_string()),
            timestamp: SgBigInt("1700000000".to_string()),
        }
    }

    fn default_sg_vault_balance_change_vault_ref() -> SgVaultBalanceChangeVault {
        SgVaultBalanceChangeVault {
            id: SgBytes("0xVaultIdForBalanceChange".to_string()),
            vault_id: SgBigInt("12345".to_string()),
            token: default_sg_erc20(),
        }
    }

    fn default_sg_vault_balance_change_unwrapped() -> SgVaultBalanceChangeUnwrapped {
        SgVaultBalanceChangeUnwrapped {
            __typename: "Deposit".to_string(),
            amount: SgBigInt("500000000000000000".to_string()),
            new_vault_balance: SgBigInt("1500000000000000000".to_string()),
            old_vault_balance: SgBigInt("1000000000000000000".to_string()),
            vault: default_sg_vault_balance_change_vault_ref(),
            timestamp: SgBigInt("1700000100".to_string()),
            transaction: default_sg_transaction(),
            orderbook: default_sg_orderbook(),
        }
    }

    fn assert_sg_vault_balance_change_unwrapped_eq(
        actual: &SgVaultBalanceChangeUnwrapped,
        expected: &SgVaultBalanceChangeUnwrapped,
    ) {
        assert_eq!(actual.__typename, expected.__typename);
        assert_eq!(actual.amount, expected.amount);
        assert_eq!(actual.new_vault_balance, expected.new_vault_balance);
        assert_eq!(actual.old_vault_balance, expected.old_vault_balance);
        assert_eq!(actual.vault.id, expected.vault.id);
        assert_eq!(actual.vault.vault_id, expected.vault.vault_id);
        assert_eq!(actual.vault.token.id, expected.vault.token.id);
        assert_eq!(actual.timestamp, expected.timestamp);
        assert_eq!(actual.transaction.id, expected.transaction.id);
        assert_eq!(actual.orderbook.id, expected.orderbook.id);
    }

    #[tokio::test]
    async fn test_vault_detail_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id_str = "0xExistingVaultId";
        let vault_id = Id::new(vault_id_str);
        let expected_vault = default_sg_vault();

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vault": expected_vault}}));
        });

        let result = client.vault_detail(vault_id).await;
        assert!(result.is_ok());
        let vault = result.unwrap();
        assert_sg_vault_eq(&vault, &expected_vault);
    }

    #[tokio::test]
    async fn test_vault_detail_not_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id_str = "0xNonExistentVaultId";
        let vault_id = Id::new(vault_id_str);

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"vault": null}}));
        });

        let result = client.vault_detail(vault_id).await;
        assert!(matches!(result, Err(OrderbookSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_vault_detail_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id = Id::new("0xAnyVaultId");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.vault_detail(vault_id).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_vaults_list_no_filters() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: false,
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };
        let expected_vaults = vec![default_sg_vault(), default_sg_vault()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"skip\":0")
                .body_contains("\"first\":10");
            then.status(200)
                .json_body(json!({"data": {"vaults": expected_vaults}}));
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let vaults = result.unwrap();
        assert_eq!(vaults.len(), expected_vaults.len());
        for (actual, expected) in vaults.iter().zip(expected_vaults.iter()) {
            assert_sg_vault_eq(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_vaults_list_with_owner_filter() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let owner_address = SgBytes("0xSpecificOwner".to_string());
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![owner_address.clone()],
            hide_zero_balance: false,
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 5,
        };
        let expected_vaults = vec![default_sg_vault()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"owner_in\":[\"0xSpecificOwner\"]");
            then.status(200)
                .json_body(json!({"data": {"vaults": expected_vaults}}));
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let vaults = result.unwrap();
        assert_eq!(vaults.len(), expected_vaults.len());
    }

    #[tokio::test]
    async fn test_vaults_list_with_hide_zero_balance_true() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: true,
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 5,
        };
        let expected_vaults = vec![default_sg_vault()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"balance_gt\":\"0\"");
            then.status(200)
                .json_body(json!({"data": {"vaults": expected_vaults}}));
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let vaults = result.unwrap();
        assert_eq!(vaults.len(), expected_vaults.len());
    }

    #[tokio::test]
    async fn test_vaults_list_pagination() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: false,
        };
        let pagination_args = SgPaginationArgs {
            page: 2,
            page_size: 3,
        };
        let expected_vaults = vec![default_sg_vault()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"skip\":3")
                .body_contains("\"first\":3");
            then.status(200)
                .json_body(json!({"data": {"vaults": expected_vaults}}));
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vaults_list_empty_result() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: false,
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"vaults": []}}));
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_vaults_list_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: false,
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.vaults_list(filter_args, pagination_args).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_vaults_list_all_multiple_pages() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vaults_page1: Vec<SgVault> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|_| default_sg_vault())
            .collect();
        let vaults_page2: Vec<SgVault> = (0..50).map(|_| default_sg_vault()).collect();

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains("\"skip\":0")
                .body_contains("\"balance_gt\":\"0\"");
            then.status(200)
                .json_body(json!({"data": {"vaults": vaults_page1}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains("\"balance_gt\":\"0\"");
            then.status(200)
                .json_body(json!({"data": {"vaults": vaults_page2}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE * 2))
                .body_contains("\"balance_gt\":\"0\"");
            then.status(200).json_body(json!({"data": {"vaults": []}}));
        });

        let result = client.vaults_list_all().await;
        assert!(result.is_ok());
        let vaults = result.unwrap();
        assert_eq!(vaults.len(), ALL_PAGES_QUERY_PAGE_SIZE as usize + 50);
    }

    #[tokio::test]
    async fn test_vaults_list_all_no_vaults() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains("\"skip\":0")
                .body_contains("\"balance_gt\":\"0\"");
            then.status(200).json_body(json!({"data": {"vaults": []}}));
        });
        let result = client.vaults_list_all().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_vaults_list_all_network_error_on_page() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vaults_page1: Vec<SgVault> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|_| default_sg_vault())
            .collect();

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"vaults": vaults_page1}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE));
            then.status(500);
        });

        let result = client.vaults_list_all().await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id_str = "0xVaultForBalanceChanges";
        let vault_id = Id::new(vault_id_str);
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };
        let expected_changes = vec![default_sg_vault_balance_change_unwrapped()];

        sg_server.mock(|when, then| {
            when.method(POST).path("/").body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": expected_changes}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST).path("/").body_contains("\"skip\":200");
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": []}}));
        });

        let result = client
            .vault_balance_changes_list(vault_id.clone(), pagination_args)
            .await;
        assert!(result.is_ok(), "Result was: {:?}", result.err());
        let changes = result.unwrap();
        assert_eq!(changes.len(), expected_changes.len());
        for (actual, expected) in changes.iter().zip(expected_changes.iter()) {
            assert_sg_vault_balance_change_unwrapped_eq(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_empty_result() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id = Id::new("0xVaultWithNoChanges");
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": []}}));
        });

        let result = client
            .vault_balance_changes_list(vault_id, pagination_args)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id = Id::new("0xAnyVaultId");
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client
            .vault_balance_changes_list(vault_id, pagination_args)
            .await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::PaginationClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_all_multiple_pages() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id_str = "0xVaultForBalanceChangesAll";
        let vault_id = Id::new(vault_id_str);

        let changes_page1: Vec<SgVaultBalanceChangeUnwrapped> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|_| default_sg_vault_balance_change_unwrapped())
            .collect();
        let changes_page2: Vec<SgVaultBalanceChangeUnwrapped> = (0..30)
            .map(|_| default_sg_vault_balance_change_unwrapped())
            .collect();

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", vault_id_str))
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": changes_page1}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", vault_id_str))
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE));
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": changes_page2}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", vault_id_str))
                .body_contains(format!("\"first\":{}", ALL_PAGES_QUERY_PAGE_SIZE))
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE * 2));
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": []}}));
        });

        let result = client
            .vault_balance_changes_list_all(vault_id.clone())
            .await;
        assert!(result.is_ok(), "Result was: {:?}", result.err());
        let changes = result.unwrap();
        assert_eq!(changes.len(), ALL_PAGES_QUERY_PAGE_SIZE as usize + 30);
    }

    #[tokio::test]
    async fn test_vault_balance_changes_list_all_no_changes() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let vault_id = Id::new("0xVaultWithNoChangesAll");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaultBalanceChanges": []}}));
        });

        let result = client.vault_balance_changes_list_all(vault_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
