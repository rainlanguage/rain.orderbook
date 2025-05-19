use super::*;

impl OrderbookSubgraphClient {
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
}
