#[cfg(test)]
mod tests {
    use alloy::sol_types::SolCall;
    use alloy::{
        network::TransactionBuilder,
        primitives::{utils::parse_ether, U256},
        rpc::types::TransactionRequest,
    };
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_test_fixtures::{LocalEvm, Orderbook::*};

    #[tokio::test]
    async fn test_post_task_set() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;

        let token1_holder = local_evm.signer_wallets[0].default_signer().address();

        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

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
    eth:
        network: some-key
        address: {token2}
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: some-key
        address: {token1}
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: eth
        outputs:
            - token: dai
              vault-id: 0x01
scenarios:
    some-key:
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#calculate-io
using-words-from {orderbook_subparser}
amount price: get("amount") 52;
#handle-add-order
:set("amount" 100);
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            orderbook_subparser = local_evm.orderbook_subparser.address(),
            deployer = local_evm.deployer.address(),
            token1 = token1.address(),
            token2 = token2.address(),
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.dotrain_yaml.get_deployment("some-key").unwrap();
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
                parse_ether("1000").unwrap(),
                U256::from(1),
            )
            .await
            .0
            .order;

        let quote = local_evm
            .call_contract(orderbook.quote(Quote {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }))
            .await
            .unwrap()
            .unwrap();

        let amount = quote._1;
        let price = quote._2;

        assert_eq!(amount, parse_ether("100").unwrap());
        assert_eq!(price, parse_ether("52").unwrap());
    }

    #[tokio::test]
    async fn test_post_task_revert() {
        let local_evm = LocalEvm::new().await;
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
    eth:
        network: some-key
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: some-key
        address: 0xabc0000000000000000000000000000000000004
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: eth
            - token: dai
        outputs:
            - token: dai
scenarios:
    some-key:
deployments:
    some-key:
        scenario: some-key
        order: some-key
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
        let deployment = order.dotrain_yaml.get_deployment("some-key").unwrap();
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

        let res = local_evm
            .send_transaction(tx)
            .await
            .expect_err("Transaction should have reverted");

        assert!(res.to_string().contains("should fail"));
    }
}
