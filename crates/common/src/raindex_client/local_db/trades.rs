use super::super::trades::{RaindexTrade, RaindexTradeWithOwner};
use super::super::RaindexError;
use super::query::fetch_transaction_trades::fetch_transaction_trades;
use super::LocalDb;
use crate::local_db::OrderbookIdentifier;
use alloy::primitives::B256;

pub struct LocalDbTrades<'a> {
    pub(crate) db: &'a LocalDb,
}

impl<'a> LocalDbTrades<'a> {
    pub(crate) fn new(db: &'a LocalDb) -> Self {
        Self { db }
    }

    pub async fn get_by_tx_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        tx_hash: B256,
    ) -> Result<Vec<RaindexTradeWithOwner>, RaindexError> {
        let local_trades = fetch_transaction_trades(self.db, ob_id, tx_hash).await?;

        local_trades
            .into_iter()
            .map(|trade| {
                let owner = trade.order_owner;
                let trade = RaindexTrade::try_from_local_db_trade(ob_id.chain_id, trade)?;
                Ok(RaindexTradeWithOwner {
                    order_owner: owner,
                    trade,
                })
            })
            .collect()
    }
}
