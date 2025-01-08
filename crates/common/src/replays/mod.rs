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
    use crate::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use alloy::{
        network::TransactionBuilder,
        primitives::{
            utils::{parse_ether, parse_units},
            Bytes, U256,
        },
        rpc::types::TransactionRequest,
        sol_types::SolCall,
    };
    use rain_orderbook_test_fixtures::{ContractTxHandler, LocalEvm, Orderbook};

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_trade_replayer() {
        let mut local_evm = LocalEvm::new().await;

        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token2_holder = local_evm.signer_wallets[1].default_signer().address();

        let token1 = local_evm
            .deploy_new_token(
                "T1",
                "T1",
                18,
                parse_units("100", 18).unwrap().into(),
                token1_holder,
            )
            .await;
        let token2 = local_evm
            .deploy_new_token(
                "T2",
                "T2",
                18,
                parse_units("100", 18).unwrap().into(),
                token2_holder,
            )
            .await;
        let orderbook = &local_evm.orderbook;

        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: {deployer}
tokens:
    eth:
        network: polygon
        address: {token2}
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: polygon
        address: {token1}
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    polygon:
        address: {orderbook}
orders:
    polygon:
        inputs:
            - token: eth
        outputs:
            - token: dai
              vault-id: 0x01
scenarios:
    polygon:
deployments:
    polygon:
        scenario: polygon
        order: polygon
---
#calculate-io
amount price: 2 1;
#handle-add-order
:;
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            deployer = local_evm.deployer.address(),
            token1 = token1.address(),
            token2 = token2.address(),
        );

        // add order
        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.dotrain_yaml.get_deployment("polygon").unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(local_evm.url())
            .await
            .unwrap()
            .abi_encode();
        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address())
            .with_from(token1_holder);
        local_evm.send_transaction(tx).await.unwrap();

        let filter = orderbook.AddOrderV2_filter();
        let logs = filter.query().await.unwrap();
        let order = logs[0].0.order.clone();

        // approve and deposit Token1
        token1
            .approve(*orderbook.address(), parse_ether("1000").unwrap())
            .do_send(&local_evm)
            .await
            .unwrap();
        orderbook
            .deposit2(
                *token1.address(),
                U256::from(0x01),
                parse_ether("10").unwrap(),
                vec![],
            )
            .do_send(&local_evm)
            .await
            .unwrap();

        // approve T2 spending for token2 holder for orderbook
        token2
            .approve(*orderbook.address(), parse_ether("1000").unwrap())
            .from(token2_holder)
            .do_send(&local_evm)
            .await
            .unwrap();
        // take order from token2 holder
        let config = Orderbook::TakeOrdersConfigV3 {
            orders: vec![Orderbook::TakeOrderConfigV3 {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }],
            maximumIORatio: U256::MAX,
            maximumInput: U256::MAX,
            minimumInput: U256::from(1),
            data: Bytes::new(),
        };
        let tx = orderbook
            .takeOrders2(config)
            .from(token2_holder)
            .do_send(&local_evm)
            .await
            .unwrap();

        let mut replayer = TradeReplayer::new(NewTradeReplayer {
            fork_url: local_evm.url().as_str().try_into().unwrap(),
        })
        .await
        .unwrap();

        let res = replayer.replay_tx(tx.transaction_hash).await.unwrap();

        let vec = vec![1000000000000000000u128, 2000000000000000000u128];

        let expected_stack: Vec<U256> = vec.into_iter().map(U256::from).collect();

        assert_eq!(res.traces[1].stack, expected_stack);
        assert_eq!(res.traces.len(), 2);
    }
}
