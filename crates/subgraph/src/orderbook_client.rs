use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PaginationClient, PaginationClientError, SgPaginationArgs};
use crate::performance::vol::{get_vaults_vol, VaultVolume};
use crate::performance::OrderPerformance;
use crate::types::add_order::{SgTransactionAddOrdersQuery, TransactionAddOrdersVariables};
use crate::types::common::*;
use crate::types::order::{
    SgBatchOrderDetailQuery, SgBatchOrderDetailQueryVariables, SgOrderDetailByHashQuery,
    SgOrderDetailByHashQueryVariables, SgOrderDetailByIdQuery, SgOrderIdList, SgOrdersListQuery,
};
use crate::types::order_trade::{SgOrderTradeDetailQuery, SgOrderTradesListQuery};
use crate::types::remove_order::{
    SgTransactionRemoveOrdersQuery, TransactionRemoveOrdersVariables,
};
use crate::types::transaction::SgTransactionDetailQuery;
use crate::types::vault::{SgVaultDetailQuery, SgVaultsListQuery};
use crate::vault_balance_changes_query::VaultBalanceChangesListPageQueryClient;
use cynic::Id;
use reqwest::Url;
use std::num::ParseIntError;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::{JsError, JsValue};

const ALL_PAGES_QUERY_PAGE_SIZE: u16 = 200;

#[derive(Error, Debug)]
pub enum OrderbookSubgraphClientError {
    #[error(transparent)]
    CynicClientError(#[from] CynicClientError),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error("Request timed out")]
    RequestTimedOut,
    #[error(transparent)]
    PaginationClientError(#[from] PaginationClientError),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    PerformanceError(#[from] crate::performance::PerformanceError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] wasm_bindgen_utils::prelude::serde_wasm_bindgen::Error),
    #[error("Failed to extend the order detail")]
    OrderDetailExtendError,
}

#[cfg(target_family = "wasm")]
impl From<OrderbookSubgraphClientError> for JsValue {
    fn from(value: OrderbookSubgraphClientError) -> Self {
        JsError::new(&value.to_string()).into()
    }
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
    pub async fn order_detail(&self, id: Id) -> Result<SgOrder, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgOrderDetailByIdQuery, SgIdQueryVariables>(SgIdQueryVariables { id: &id })
            .await?;
        let order = data.order.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order)
    }

    /// Fetch batch orders given their order id
    pub async fn batch_order_detail(
        &self,
        id_list: Vec<SgBytes>,
    ) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgBatchOrderDetailQuery, SgBatchOrderDetailQueryVariables>(
                SgBatchOrderDetailQueryVariables {
                    id_list: SgOrderIdList { id_in: id_list },
                },
            )
            .await?;

        Ok(data.orders)
    }

    /// Fetch all orders, paginated
    pub async fn orders_list(
        &self,
        filter_args: SgOrdersListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);

        let filters = if !filter_args.owners.is_empty()
            || filter_args.active.is_some()
            || filter_args.order_hash.is_some()
        {
            Some(SgOrdersListQueryFilters {
                owner_in: filter_args.owners,
                active: filter_args.active,
                order_hash: filter_args.order_hash,
            })
        } else {
            None
        };

        let variables = SgOrdersListQueryVariables {
            first: pagination_variables.first,
            skip: pagination_variables.skip,
            filters,
        };

        let data = self
            .query::<SgOrdersListQuery, SgOrdersListQueryVariables>(variables)
            .await?;

        Ok(data.orders)
    }

    /// Fetch all pages of orders_list query
    pub async fn orders_list_all(&self) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .orders_list(
                    SgOrdersListFilterArgs {
                        owners: vec![],
                        active: None,
                        order_hash: None,
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

    /// Fetch single order take
    pub async fn order_trade_detail(
        &self,
        id: Id,
    ) -> Result<SgTrade, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgOrderTradeDetailQuery, SgIdQueryVariables>(SgIdQueryVariables { id: &id })
            .await?;
        let order_take = data.trade.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order_take)
    }

    /// Fetch all order takes paginated for a single order
    pub async fn order_trades_list(
        &self,
        order_id: cynic::Id,
        pagination_args: SgPaginationArgs,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<SgTrade>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);
        let data = self
            .query::<SgOrderTradesListQuery, SgPaginationWithTimestampQueryVariables>(
                SgPaginationWithTimestampQueryVariables {
                    id: SgBytes(order_id.inner().to_string()),
                    first: pagination_variables.first,
                    skip: pagination_variables.skip,
                    timestamp_gte: Some(
                        start_timestamp
                            .map_or(SgBigInt("0".to_string()), |v| SgBigInt(v.to_string())),
                    ),
                    timestamp_lte: Some(
                        end_timestamp
                            .map_or(SgBigInt(u64::MAX.to_string()), |v| SgBigInt(v.to_string())),
                    ),
                },
            )
            .await?;

        Ok(data.trades)
    }

    /// Fetch all pages of order_takes_list query
    pub async fn order_trades_list_all(
        &self,
        order_id: cynic::Id,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<SgTrade>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .order_trades_list(
                    order_id.clone(),
                    SgPaginationArgs {
                        page,
                        page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                    },
                    start_timestamp,
                    end_timestamp,
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

    /// Fetch all pages of order_takes_list query and calculate vaults' vol
    pub async fn order_vaults_volume(
        &self,
        order_id: cynic::Id,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<VaultVolume>, OrderbookSubgraphClientError> {
        let trades = self
            .order_trades_list_all(order_id, start_timestamp, end_timestamp)
            .await?;
        Ok(get_vaults_vol(&trades)?)
    }

    /// Fetches order data and measures an order's detailed performance (apy and vol)
    pub async fn order_performance(
        &self,
        order_id: cynic::Id,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, OrderbookSubgraphClientError> {
        let order = self.order_detail(order_id.clone()).await?;
        let trades = self
            .order_trades_list_all(order_id, start_timestamp, end_timestamp)
            .await?;
        Ok(OrderPerformance::measure(
            &order,
            &trades,
            start_timestamp,
            end_timestamp,
        )?)
    }

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

    pub async fn transaction_detail(
        &self,
        id: Id,
    ) -> Result<SgTransaction, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgTransactionDetailQuery, SgIdQueryVariables>(SgIdQueryVariables { id: &id })
            .await?;
        let transaction = data
            .transaction
            .ok_or(OrderbookSubgraphClientError::Empty)?;
        Ok(transaction)
    }

    /// Fetch all add orders for a given transaction
    pub async fn transaction_add_orders(
        &self,
        id: Id,
    ) -> Result<Vec<SgAddOrderWithOrder>, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgTransactionAddOrdersQuery, TransactionAddOrdersVariables>(
                TransactionAddOrdersVariables {
                    id: SgBytes(id.inner().to_string()),
                },
            )
            .await?;

        if data.add_orders.is_empty() {
            return Err(OrderbookSubgraphClientError::Empty);
        }

        Ok(data.add_orders)
    }

    /// Fetch all remove orders for a given transaction
    pub async fn transaction_remove_orders(
        &self,
        id: Id,
    ) -> Result<Vec<SgRemoveOrderWithOrder>, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgTransactionRemoveOrdersQuery, TransactionRemoveOrdersVariables>(
                TransactionRemoveOrdersVariables {
                    id: SgBytes(id.inner().to_string()),
                },
            )
            .await?;

        if data.remove_orders.is_empty() {
            return Err(OrderbookSubgraphClientError::Empty);
        }

        Ok(data.remove_orders)
    }

    /// Fetch single order given its hash
    pub async fn order_detail_by_hash(
        &self,
        hash: SgBytes,
    ) -> Result<SgOrder, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgOrderDetailByHashQuery, SgOrderDetailByHashQueryVariables>(
                SgOrderDetailByHashQueryVariables { hash },
            )
            .await?;
        let order = data
            .orders
            .first()
            .ok_or(OrderbookSubgraphClientError::Empty)?;
        Ok(order.clone())
    }
}
