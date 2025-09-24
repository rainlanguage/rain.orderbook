use crate::error::CommandResult;
use alloy::primitives::{Address, U256};
use rain_orderbook_bindings::IOrderBookV5::QuoteV2;
use rain_orderbook_common::fuzz::{RainEvalResults, RainEvalResultsTable};
use rain_orderbook_quote::{NewQuoteDebugger, QuoteDebugger, QuoteDebuggerError, QuoteTarget};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use std::str::FromStr;

#[tauri::command]
pub async fn debug_order_quote(
    order: SgOrder,
    rpcs: Vec<String>,
    input_io_index: u32,
    output_io_index: u32,
    block_number: Option<u32>,
) -> CommandResult<(RainEvalResultsTable, Option<String>)> {
    let quote_target = QuoteTarget {
        orderbook: Address::from_str(&order.orderbook.id.0)?,
        quote_config: QuoteV2 {
            order: order.try_into()?,
            inputIOIndex: U256::from(input_io_index),
            outputIOIndex: U256::from(output_io_index),
            signedContext: vec![],
        },
    };

    let mut err: Option<QuoteDebuggerError> = None;
    for rpc in rpcs {
        match QuoteDebugger::new(NewQuoteDebugger {
            fork_url: rpc.parse()?,
            fork_block_number: block_number.map(|s| s.into()),
        })
        .await
        {
            Ok(mut debugger) => {
                let res = debugger.debug(quote_target).await?;
                let eval_res: RainEvalResults = vec![res.0.clone()].into();

                return Ok((
                    eval_res.into_flattened_table(),
                    res.1.map(|v| match v {
                        Ok(e) => e.to_string(),
                        Err(e) => e.to_string(),
                    }),
                ));
            }
            Err(e) => {
                err = Some(e);
            }
        }
    }

    // if we are here, we have tried all rpcs and failed
    Err(err.unwrap().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::B256;
    use alloy::{
        hex::encode_prefixed,
        primitives::utils::parse_ether,
        sol_types::{SolCall, SolValue},
    };
    use httpmock::MockServer;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_subgraph_client::types::common::{SgBigInt, SgBytes, SgOrder, SgOrderbook};
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_debug_order_quote() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

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
    t1:
        network: some-key
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
    t2:
        network: some-key
        address: {token1}
        decimals: 18
        label: Token1
        symbol: token1
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: t1
        outputs:
            - token: t2
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
amount price: 16 52;
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

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment, None)
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
                parse_ether("1000").unwrap(),
                18,
                B256::from(U256::from(1)),
            )
            .await
            .0
            .order;

        let sg_order = SgOrder {
            id: SgBytes("0x01".to_string()),
            orderbook: SgOrderbook {
                id: SgBytes(orderbook.address().to_string()),
            },
            order_bytes: SgBytes(encode_prefixed(order.abi_encode())),
            order_hash: SgBytes("0x01".to_string()),
            owner: SgBytes("0x0000000000000000000000000000000000000001".to_string()),
            outputs: vec![],
            inputs: vec![],
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: SgBigInt(0.to_string()),
            trades: vec![],
            remove_events: vec![],
        };

        let input_io_index = 0;
        let output_io_index = 0;

        const CHAIN_ID_1_ORDERBOOK_ADDRESS: &str = "0x1234567890123456789012345678901234567890";

        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "orders": [json!({
                        "id": "0x46891c626a8a188610b902ee4a0ce8a7e81915e1b922584f8168d14525899dfb",
                        "orderBytes": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000005f6c104ca9812ef91fe2e26a2e7187b92d3b0e800000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000022009cd210f509c66e18fab61fd30f76fb17c6c6cd09f0972ce0815b5b7630a1b050000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae600000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000075000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372",
                        "orderHash": "0x283508c8f56f4de2f21ee91749d64ec3948c16bc6b4bfe4f8d11e4e67d76f4e0",
                        "owner": "0x0000000000000000000000000000000000000000",
                        "outputs": [
                          {
                            "id": "0x0000000000000000000000000000000000000000",
                            "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                            "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                            "balance": "987000000000000000",
                            "token": {
                              "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                              "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                              "name": "Staked FLR",
                              "symbol": "sFLR",
                              "decimals": "18"
                            },
                            "orderbook": {
                              "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                            },
                            "ordersAsOutput": [],
                            "ordersAsInput": [],
                            "balanceChanges": []
                          }
                        ],
                        "inputs": [
                          {
                            "id": "0x0000000000000000000000000000000000000000",
                            "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                            "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                            "balance": "797990000000000000",
                            "token": {
                              "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                              "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                              "name": "WFLR",
                              "symbol": "WFLR",
                              "decimals": "18"
                            },
                            "orderbook": {
                              "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                            },
                            "ordersAsOutput": [],
                            "ordersAsInput": [],
                            "balanceChanges": []
                          },
                        ],
                        "orderbook": {
                          "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                        },
                        "active": true,
                        "timestampAdded": "1739448802",
                        "meta": null,
                        "addEvents": [],
                        "trades": [],
                        "removeEvents": []
                      })]
                }
            }));
        });

        let result = debug_order_quote(
            sg_order,
            vec![local_evm.url()],
            input_io_index,
            output_io_index,
            None,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.rows[0], [U256::from(16), U256::from(52)]);
    }
}
