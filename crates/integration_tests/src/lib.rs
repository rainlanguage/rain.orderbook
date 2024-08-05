use alloy::sol;

sol!(
    #![sol(all_derives = true, rpc = true)]
    Orderbook, "../../out/OrderBook.sol/OrderBook.json"
);

sol!(
    #![sol(all_derives = true, rpc = true)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use alloy::providers::ext::AnvilApi;
    use alloy::{
        network::{EthereumWallet, TransactionBuilder},
        node_bindings::Anvil,
        primitives::{utils::parse_ether, U256},
        providers::{Provider, ProviderBuilder},
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use alloy_sol_types::SolCall;
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use Orderbook::*;

    #[tokio::test]
    async fn test_post_task_set() {
        let dai_holder = "0x788F5c68331a773f226747edCef20Ce60E9d78E7"
            .parse::<Address>()
            .unwrap();

        const DAI_ADDRESS: &str = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063";

        let provider = ProviderBuilder::new().on_anvil_with_config(|anvil| {
            anvil.fork(rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL)
        });
        provider.anvil_auto_impersonate_account(true).await.unwrap();

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
        address: 0xB3aC858bEAf7814892d3946A8C109A7D701DF8E7
tokens:
    eth:
        network: polygon
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: polygon
        address: {dai}
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    polygon:
        address: 0x1234567890123456789012345678901234567891
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
using-words-from 0xe80e7438ce6b1055c8e9CDE1b6336a4F9D53C666
amount price: get("amount") 52;
#post-add-order
:set("amount" 100);
#handle-io
:;
"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL,
            dai = DAI_ADDRESS
        );

        let orderbook = Orderbook::deploy(&provider).await.unwrap();

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();

        let deployment = order.config.deployments["polygon"].as_ref().clone();

        let args = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap();

        let call = args
            .try_into_call(rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL.into())
            .await
            .unwrap();

        let calldata = call.abi_encode();

        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address())
            .with_from(dai_holder);

        let _tx_hash = provider
            .send_transaction(tx)
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        let filter = orderbook.AddOrderV2_filter();
        let logs = filter.query().await.unwrap();
        let order = logs[0].0.order.clone();

        // approve and deposit DAI
        let dai = ERC20::new(DAI_ADDRESS.parse::<Address>().unwrap(), &provider);
        dai.approve(*orderbook.address(), parse_ether("1000").unwrap())
            .from(dai_holder)
            .send()
            .await
            .unwrap()
            .watch()
            .await
            .unwrap();

        orderbook
            .deposit2(
                DAI_ADDRESS.parse::<Address>().unwrap(),
                U256::from(0x01),
                parse_ether("1000").unwrap(),
                vec![],
            )
            .from(dai_holder)
            .send()
            .await
            .unwrap()
            .watch()
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
        // Spin up a local Anvil node.
        // Ensure `anvil` is available in $PATH.
        let anvil = Anvil::new()
            .fork(rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL)
            .try_spawn()
            .unwrap();

        // Set up signer from the first default Anvil account (Alice).
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        // Create a provider with the wallet.
        let rpc_url = anvil.endpoint().parse().unwrap();
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);

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
        address: 0xB3aC858bEAf7814892d3946A8C109A7D701DF8E7
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
        address: 0x1234567890123456789012345678901234567891
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
#post-add-order
:ensure(0 "should fail");
"#,
            rpc_url = anvil.endpoint()
        );

        let orderbook = Orderbook::deploy(provider.clone()).await.unwrap();

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();

        let deployment = order.config.deployments["polygon"].as_ref().clone();

        let args = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap();

        let call = args.try_into_call(anvil.endpoint()).await.unwrap();
        let calldata = call.abi_encode();

        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address());

        let res = provider.send_transaction(tx).await;

        match res {
            Ok(_) => panic!("Transaction should have reverted"),
            Err(e) => {
                assert!(e.to_string().contains("should fail"));
            }
        }
    }
}
