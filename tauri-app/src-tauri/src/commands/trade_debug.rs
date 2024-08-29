use alloy::primitives::{B256, U256};
use rain_orderbook_common::replays::{NewTradeReplayer, TradeReplayer};

use crate::error::CommandResult;

#[tauri::command]
pub async fn debug_trade(tx_hash: String, rpc_url: String) -> CommandResult<Vec<U256>> {
    let mut replayer: TradeReplayer = TradeReplayer::new(NewTradeReplayer {
        fork_url: rpc_url.parse().unwrap(),
    })
    .await?;
    let tx_hash = tx_hash.parse::<B256>().unwrap();
    let res = replayer.replay_tx(tx_hash).await?;
    let stack = res.traces[1].stack.iter().map(|x| x.clone()).collect();
    Ok(stack)
}

#[cfg(test)]
mod tests {
    use rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL;
    use std::str::FromStr;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_trade_replayer() {
        let tx_hash: Result<alloy::primitives::FixedBytes<32>, alloy::hex::FromHexError> =
            B256::from_str("0xceb48768613542fe2a05504200caa47dc19c4e508bd70ec3b18e648eebf58177");

        let res = debug_trade(
            tx_hash.unwrap().to_string(),
            CI_DEPLOY_POLYGON_RPC_URL.to_string(),
        )
        .await
        .unwrap();

        let vec = vec![
            8255747967003398332u128,
            5195342786557434200u128,
            43067440648007307827u128,
            5195342786557434200u128,
            519534278655743420u128,
            1724872944000000000000000000u128,
            1724873082000000000000000000u128,
        ];

        let expected_stack: Vec<U256> = vec.into_iter().map(U256::from).collect();

        assert_eq!(res, expected_stack);
    }
}
