use crate::local_db::query::fetch_transaction_trades::{
    build_fetch_transaction_trades_stmt, LocalDbTransactionTrade,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::OrderbookIdentifier;
use alloy::primitives::B256;

pub async fn fetch_transaction_trades<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    ob_id: &OrderbookIdentifier,
    tx_hash: B256,
) -> Result<Vec<LocalDbTransactionTrade>, LocalDbQueryError> {
    let stmt = build_fetch_transaction_trades_stmt(ob_id, tx_hash);
    exec.query_json(&stmt).await
}
