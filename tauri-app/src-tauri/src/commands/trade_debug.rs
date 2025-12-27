use alloy::primitives::B256;
use rain_orderbook_common::{
    fuzz::{RainEvalResults, RainEvalResultsTable},
    replays::{NewTradeReplayer, TradeReplayer},
};

use crate::error::{CommandError, CommandResult};

#[tauri::command]
pub async fn debug_trade(
    tx_hash: String,
    rpcs: Vec<String>,
) -> CommandResult<RainEvalResultsTable> {
    if rpcs.is_empty() {
        return Err(CommandError::MissingRpcs);
    }

    let mut last_error = None;
    for rpc in rpcs {
        match TradeReplayer::new(NewTradeReplayer {
            fork_url: rpc.parse()?,
        })
        .await
        {
            Ok(mut replayer) => {
                let tx_hash = tx_hash.parse::<B256>()?;
                let res: RainEvalResults = vec![replayer.replay_tx(tx_hash).await?].into();
                return Ok(res.into_flattened_table());
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    // If we get here, all rpcs failed
    Err(CommandError::TradeReplayerError(last_error.unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        primitives::{
            utils::{parse_ether, parse_units},
            Bytes, U256,
        },
        sol_types::SolCall,
    };
    use rain_math_float::Float;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_test_fixtures::{LocalEvm, Orderbook};
    use std::str::FromStr;

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
                parse_units("100000", 18).unwrap().into(),
                token1_holder,
            )
            .await;
        let token2 = local_evm
            .deploy_new_token(
                "T2",
                "T2",
                18,
                parse_units("100000", 18).unwrap().into(),
                token2_holder,
            )
            .await;
        let orderbook = &local_evm.orderbook;

        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
deployers:
    some-key:
        address: {deployer}
tokens:
    t2:
        network: some-key
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
    t1:
        network: some-key
        address: {token1}
        decimals: 18
        label: Token1
        symbol: Token1
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: t2
        outputs:
            - token: t1
              vault-id: 0x01
scenarios:
    some-key:
        deployer: some-key
        bindings:
            key: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key !Test binding
#calculate-io
amount price: 7 4;
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
            spec_version = SpecVersion::current(),
        );

        // add order
        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(vec![local_evm.url()])
            .await
            .unwrap()
            .abi_encode();

        let order = local_evm
            .add_order_and_deposit(
                &calldata,
                token1_holder,
                *token1.address(),
                parse_ether("100").unwrap(),
                18,
                B256::from(U256::from(1)),
            )
            .await
            .0
            .order;

        // approve T2 spending for token2 holder for orderbook
        local_evm
            .send_transaction(
                token2
                    .approve(*orderbook.address(), parse_ether("100").unwrap())
                    .from(token2_holder)
                    .into_transaction_request(),
            )
            .await
            .unwrap();

        // TODO: Uncomment this when we have MAX for Float
        // let max_float = Float::pack_lossless(I224::MAX, 1).unwrap().get_inner();
        let max_float = Float::from_fixed_decimal(
            U256::from_str("13479973333575319897333507543509815336818572211270286240551805124607")
                .unwrap(),
            1,
        )
        .unwrap()
        .get_inner();
        let one_float = Float::parse("1".to_string()).unwrap().get_inner();

        let config = Orderbook::TakeOrdersConfigV5 {
            orders: vec![Orderbook::TakeOrderConfigV4 {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }],
            maximumIORatio: max_float,
            maximumIO: max_float,
            minimumIO: one_float,
            IOIsInput: true,
            data: Bytes::new(),
        };

        let tx = local_evm
            .send_transaction(
                orderbook
                    .takeOrders3(config)
                    .from(token2_holder)
                    .into_transaction_request(),
            )
            .await
            .unwrap();

        let res = debug_trade(tx.transaction_hash.to_string(), vec![local_evm.url()])
            .await
            .unwrap();

        let expected_stack = vec![U256::from(7), U256::from(4)];

        assert_eq!(res.rows[0], expected_stack);
    }
}
