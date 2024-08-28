use alloy::primitives::{B256, U256};
use rain_orderbook_common::replays::{NewTradeReplayer, TradeReplayer};

use crate::error::CommandResult;

#[tauri::command]
pub async fn debug_trade(tx_hash: String, rpc_url: String) -> CommandResult<Vec<U256>> {
    println!("debug_trade");
    let mut replayer: TradeReplayer = TradeReplayer::new(NewTradeReplayer {
        fork_url: rpc_url.parse().unwrap(),
    })
    .await?;
    let tx_hash = tx_hash.parse::<B256>().unwrap();
    let res = replayer.replay_tx(tx_hash).await?;
    let stack = res.traces[1].stack.iter().map(|x| x.clone()).collect();
    print!("{:?}", stack);
    Ok(stack)
}
