use super::*;

impl OrderbookSubgraphClient {
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
}
