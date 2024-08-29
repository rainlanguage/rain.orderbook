use alloy::primitives::B256;
use rain_interpreter_eval::{
    fork::{Forker, NewForkedEvm},
    trace::RainEvalResult,
};
use url::Url;

pub struct NewTradeReplayer {
    pub fork_url: Url,
}
pub struct TradeReplayer {
    forker: Forker,
}

#[derive(Debug, thiserror::Error)]
pub enum TradeReplayerError {
    #[error("Forker error: {0}")]
    ForkerError(#[from] rain_interpreter_eval::error::ForkCallError),
}

impl TradeReplayer {
    pub async fn new(args: NewTradeReplayer) -> Result<Self, TradeReplayerError> {
        let forker = Forker::new_with_fork(
            NewForkedEvm {
                fork_url: args.fork_url.to_string(),
                fork_block_number: None,
            },
            None,
            None,
        )
        .await?;

        Ok(Self { forker })
    }

    pub async fn replay_tx(&mut self, tx_hash: B256) -> Result<RainEvalResult, TradeReplayerError> {
        let res = self.forker.replay_transaction(tx_hash).await?;
        Ok(res.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;
    use rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL;
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_trade_replayer() {
        let mut replayer = TradeReplayer::new(NewTradeReplayer {
            fork_url: Url::from_str(CI_DEPLOY_POLYGON_RPC_URL).unwrap(),
        })
        .await
        .unwrap();

        let tx_hash: Result<alloy::primitives::FixedBytes<32>, alloy::hex::FromHexError> =
            B256::from_str("0xceb48768613542fe2a05504200caa47dc19c4e508bd70ec3b18e648eebf58177");

        let res = replayer.replay_tx(tx_hash.unwrap()).await.unwrap();

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

        assert_eq!(res.traces[1].stack, expected_stack);
        assert_eq!(res.traces.len(), 16);
    }
}
