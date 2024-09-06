use alloy::sol;

mod local_evm;
pub use local_evm::*;

sol!(
    #![sol(all_derives = true, rpc = true)]
    Orderbook, "../../out/OrderBook.sol/OrderBook.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    ERC20, "../../out/TestERC20.sol/TestERC20.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Interpreter,
    "../../out/RainterpreterNPE2.sol/RainterpreterNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Store,
    "../../out/RainterpreterStoreNPE2.sol/RainterpreterStoreNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Parser,
    "../../out/RainterpreterParserNPE2.sol/RainterpreterParserNPE2.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    Deployer,
    "../../out/RainterpreterExpressionDeployerNPE2.sol/RainterpreterExpressionDeployerNPE2.json"
);

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::sol_types::SolCall;
    use alloy::{
        network::TransactionBuilder,
        primitives::{utils::parse_ether, U256},
        rpc::types::TransactionRequest,
    };
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use Orderbook::*;

    #[tokio::test]
    async fn test_post_task_set() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let tokens = local_evm.tokens.values().collect::<Vec<_>>();
        let token1 = tokens[0].clone();
        let token2 = tokens[1].clone();
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();

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
/* using-words-from 0xe80e7438ce6b1055c8e9CDE1b6336a4F9D53C666 */
amount price: get("amount") 52;
#handle-add-order
:set("amount" 100);
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            deployer = local_evm.deployer.address(),
            token1 = token1.address(),
            token2 = token2.address(),
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.config().deployments["polygon"].as_ref().clone();
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
        local_evm
            .send_contract_call_transaction(
                token1.approve(*orderbook.address(), parse_ether("1000").unwrap()),
            )
            .await
            .unwrap();

        local_evm
            .send_contract_call_transaction(orderbook.deposit2(
                *token1.address(),
                U256::from(0x01),
                parse_ether("1000").unwrap(),
                vec![],
            ))
            .await
            .unwrap();

        let quote = orderbook
            .quote(Quote {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            })
            .call()
            .await
            .unwrap();

        let amount = quote._1;
        let price = quote._2;

        assert_eq!(amount, parse_ether("100").unwrap());
        assert_eq!(price, parse_ether("52").unwrap());
    }

    #[tokio::test]
    async fn test_post_task_revert() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
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
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: polygon
        address: 0xabc0000000000000000000000000000000000004
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
            - token: dai
        outputs:
            - token: dai
scenarios:
    polygon:
deployments:
    polygon:
        scenario: polygon
        order: polygon
---
#calculate-io
amount price: get("amount") 52;
#handle-io
:;
#handle-add-order
:ensure(0 "should fail");
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            deployer = local_evm.deployer.address(),
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.config().deployments["polygon"].as_ref().clone();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(local_evm.url())
            .await
            .unwrap()
            .abi_encode();
        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address());

        let res = local_evm.send_transaction(tx).await;

        match res {
            Ok(_) => panic!("Transaction should have reverted"),
            Err(e) => {
                assert!(e.to_string().contains("should fail"));
            }
        }
    }
}
