use super::*;

impl OrderbookSubgraphClient {
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
}
