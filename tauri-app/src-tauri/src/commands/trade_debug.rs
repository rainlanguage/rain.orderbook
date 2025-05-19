use alloy::primitives::B256;
use rain_orderbook_common::{
    fuzz::{RainEvalResults, RainEvalResultsTable},
    replays::{NewTradeReplayer, TradeReplayer},
};

use crate::error::CommandResult;

#[tauri::command]
pub async fn debug_trade(tx_hash: String, rpc_url: String) -> CommandResult<RainEvalResultsTable> {
    let mut replayer: TradeReplayer = TradeReplayer::new(NewTradeReplayer {
        fork_url: rpc_url.parse()?,
    })
    .await?;
    let tx_hash = tx_hash.parse::<B256>()?;
    let res: RainEvalResults = vec![replayer.replay_tx(tx_hash).await?].into();
    Ok(res.into_flattened_table()?)
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
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
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
networks:
    some-key:
        rpc: {rpc_url}
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
        );

        // add order
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.clone(), None)
            .await
            .unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(local_evm.url())
            .await
            .unwrap()
            .abi_encode();

        let order = local_evm
            .add_order_and_deposit(
                &calldata,
                token1_holder,
                *token1.address(),
                parse_ether("100").unwrap(),
                U256::from(1),
            )
            .await
            .0
            .order;

        // approve T2 spending for token2 holder for orderbook
        token2
            .approve(*orderbook.address(), parse_ether("100").unwrap())
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

        let res = debug_trade(tx.transaction_hash.to_string(), local_evm.url())
            .await
            .unwrap();

        let expected_stack = vec![
            U256::from(7000000000000000000u128),
            U256::from(4000000000000000000u128),
        ];

        assert_eq!(res.rows[0], expected_stack);
    }
}
