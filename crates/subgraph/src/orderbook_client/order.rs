use super::*;

impl OrderbookSubgraphClient {
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
