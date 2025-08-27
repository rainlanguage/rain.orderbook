use alloy::primitives::B256;
use rain_interpreter_eval::{
    error::ForkCallError,
    fork::{Forker, NewForkedEvm},
    trace::{RainEvalResult, RainEvalResultFromRawCallResultError},
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
    ForkerError(#[from] ForkCallError),
    #[error(transparent)]
    RainEvalResultConversion(#[from] RainEvalResultFromRawCallResultError),
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
        Ok(res.try_into()?)
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
        serde::WithOtherFields,
        sol_types::SolCall,
    };
    use rain_math_float::Float;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_test_fixtures::{LocalEvm, Orderbook};
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_trade_replayer_ok() {
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
version: {spec_version}
networks:
    polygon:
        rpcs:
            - {rpc_url}
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
        deployer: polygon
        bindings:
            key1: 10
deployments:
    polygon:
        scenario: polygon
        order: polygon
---
#key1 !Test binding
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
            spec_version = SpecVersion::current()
        );

        // add order
        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("polygon")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(vec![local_evm.url()])
            .await
            .unwrap()
            .abi_encode();
        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address())
            .with_from(token1_holder);
        local_evm
            .send_transaction(WithOtherFields::new(tx))
            .await
            .unwrap();

        let filter = orderbook.AddOrderV3_filter();
        let logs = filter.query().await.unwrap();
        let order = logs[0].0.order.clone();

        // approve and deposit Token1
        local_evm
            .send_transaction(
                token1
                    .approve(*orderbook.address(), parse_ether("1000").unwrap())
                    .into_transaction_request(),
            )
            .await
            .unwrap();

        let amount = Float::parse("10".to_string()).unwrap().get_inner();
        let tx_req = orderbook
            .deposit3(
                *token1.address(),
                B256::from(U256::from(0x01)),
                amount,
                vec![],
            )
            .into_transaction_request();
        local_evm.send_transaction(tx_req).await.unwrap();

        // approve T2 spending for token2 holder for orderbook
        let tx_req = token2
            .approve(*orderbook.address(), parse_ether("1000").unwrap())
            .from(token2_holder)
            .into_transaction_request();
        local_evm.send_transaction(tx_req).await.unwrap();

        // take order from token2 holder
        // TODO: Replace this with max_positive_value from Float when available
        let max_float = Float::from_fixed_decimal(
            U256::from_str("13479973333575319897333507543509815336818572211270286240551805124607")
                .unwrap(),
            1,
        )
        .unwrap()
        .get_inner();
        let one_float = Float::parse("1".to_string()).unwrap().get_inner();
        let config = Orderbook::TakeOrdersConfigV4 {
            orders: vec![Orderbook::TakeOrderConfigV4 {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }],
            maximumIORatio: max_float,
            maximumInput: max_float,
            minimumInput: one_float,
            data: Bytes::new(),
        };

        let tx_req = orderbook
            .takeOrders3(config)
            .from(token2_holder)
            .into_transaction_request();

        let tx = local_evm.send_transaction(tx_req).await.unwrap();

        let mut replayer = TradeReplayer::new(NewTradeReplayer {
            fork_url: local_evm.url().as_str().try_into().unwrap(),
        })
        .await
        .unwrap();

        let res = replayer.replay_tx(tx.transaction_hash).await.unwrap();

        let vec = vec![1, 2];

        let expected_stack: Vec<U256> = vec.into_iter().map(U256::from).collect();

        assert_eq!(res.traces[1].stack, expected_stack);
        assert_eq!(res.traces.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_trade_replayer_err() {
        let err = TradeReplayer::new(NewTradeReplayer {
            fork_url: "https://example.com".try_into().unwrap(),
        })
        .await;

        assert!(matches!(
            err,
            Err(TradeReplayerError::ForkerError(ForkCallError::Eyre(_)))
        ));

        let local_evm = LocalEvm::new().await;

        let mut replayer = TradeReplayer::new(NewTradeReplayer {
            fork_url: local_evm.url().as_str().try_into().unwrap(),
        })
        .await
        .unwrap();

        let res = replayer.replay_tx(B256::ZERO).await;

        assert!(matches!(
            res,
            Err(TradeReplayerError::ForkerError(
                ForkCallError::ReplayTransactionError(_)
            ))
        ));
    }
}
