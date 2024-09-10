use crate::error::CommandResult;
use alloy::primitives::{Address, U256};
use alloy_ethers_typecast::transaction::ReadableClient;
use rain_orderbook_bindings::IOrderBookV4::Quote;
use rain_orderbook_common::fuzz::{RainEvalResults, RainEvalResultsTable};
use rain_orderbook_quote::{
    BatchQuoteTarget, NewQuoteDebugger, OrderQuoteValue, QuoteDebugger, QuoteTarget,
};
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct BatchOrderQuotesResponse {
    pub pair: Pair,
    #[typeshare(typescript(type = "string"))]
    pub block_number: U256,
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    pub error: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Pair {
    pub pair_name: String,
    pub input_index: u32,
    pub output_index: u32,
}

#[tauri::command]
pub async fn batch_order_quotes(
    orders: Vec<Order>,
    block_number: Option<u64>,
    rpc_url: String,
) -> CommandResult<Vec<BatchOrderQuotesResponse>> {
    let mut results: Vec<BatchOrderQuotesResponse> = Vec::new();

    for order in &orders {
        let mut pairs: Vec<Pair> = Vec::new();
        let mut quote_targets: Vec<QuoteTarget> = Vec::new();
        let orderbook = Address::from_str(&order.orderbook.id.0)?;

        for (input_index, input) in order.inputs.iter().enumerate() {
            for (output_index, output) in order.outputs.iter().enumerate() {
                let pair_name = format!(
                    "{}/{}",
                    input.token.symbol.as_deref().unwrap_or("UNKNOWN"),
                    output.token.symbol.as_deref().unwrap_or("UNKNOWN"),
                );

                let quote = order.clone().try_into()?;
                let quote_target = QuoteTarget {
                    orderbook,
                    quote_config: Quote {
                        order: quote,
                        inputIOIndex: U256::from(input_index),
                        outputIOIndex: U256::from(output_index),
                        signedContext: vec![],
                    },
                };

                if input.token.address.0 != output.token.address.0 {
                    pairs.push(Pair {
                        pair_name,
                        input_index: input_index as u32,
                        output_index: output_index as u32,
                    });
                    quote_targets.push(quote_target);
                }
            }
        }

        let req_block_number = block_number.unwrap_or(
            ReadableClient::new_from_url(rpc_url.clone())?
                .get_block_number()
                .await?,
        );

        let quote_values = BatchQuoteTarget(quote_targets)
            .do_quote(&rpc_url, Some(req_block_number), None)
            .await;

        if let Ok(quote_values) = quote_values {
            for (quote_value_result, pair) in quote_values.into_iter().zip(pairs) {
                match quote_value_result {
                    Ok(quote_value) => {
                        results.push(BatchOrderQuotesResponse {
                            pair,
                            block_number: U256::from(req_block_number),
                            success: true,
                            data: Some(quote_value),
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(BatchOrderQuotesResponse {
                            pair,
                            block_number: U256::from(req_block_number),
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        } else if let Err(e) = quote_values {
            for pair in pairs {
                results.push(BatchOrderQuotesResponse {
                    pair,
                    block_number: U256::from(req_block_number),
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn debug_order_quote(
    order: Order,
    input_io_index: u32,
    output_io_index: u32,
    orderbook: Address,
    rpc_url: String,
) -> CommandResult<RainEvalResultsTable> {
    let quote_target = QuoteTarget {
        orderbook,
        quote_config: Quote {
            order: order.try_into()?,
            inputIOIndex: U256::from(input_io_index),
            outputIOIndex: U256::from(output_io_index),
            signedContext: vec![],
        },
    };

    let mut debugger = QuoteDebugger::new(NewQuoteDebugger {
        fork_url: rpc_url.parse()?,
    })
    .await?;

    let res: RainEvalResults = vec![debugger.debug(quote_target).await?].into();

    Ok(res.into_flattened_table()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::{utils::parse_ether, B256},
        sol_types::{SolCall, SolValue},
    };
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_test_fixtures::LocalEvm;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_debug_order_quote() {
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
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
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
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.config().deployments["some-key"].as_ref().clone();
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

        let order = Order {
            id: Bytes("0x01".to_string()),
            orderbook: Orderbook {
                id: Bytes(orderbook.address().to_string()),
            },
            order_bytes: Bytes(encode_prefixed(order.abi_encode())),
            order_hash: Bytes("0x01".to_string()),
            owner: Bytes("0x01".to_string()),
            outputs: vec![],
            inputs: vec![],
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: BigInt(0.to_string()),
            trades: vec![]
        };

        let input_io_index = 0;
        let output_io_index = 0;

        let rpc_url = local_evm.url();

        let result = debug_order_quote(
            order,
            input_io_index,
            output_io_index,
            *orderbook.address(),
            rpc_url,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().rows[0],
            [parse_ether("16").unwrap(), parse_ether("52").unwrap()]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_batch_order_quotes_block() {
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
        symbol: token1
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
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
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
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.config().deployments["some-key"].as_ref().clone();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(local_evm.url())
            .await
            .unwrap()
            .abi_encode();

        let order = encode_prefixed(
            local_evm
                .add_order_and_deposit(
                    &calldata,
                    token1_holder,
                    *token1.address(),
                    parse_ether("1000").unwrap(),
                    U256::from(1),
                )
                .await
                .0
                .order
                .abi_encode(),
        );

        let inputs = vec![Vault {
            id: Bytes(B256::random().to_string()),
            token: Erc20 {
                id: Bytes(token2.address().to_string()),
                address: Bytes(token2.address().to_string()),
                name: Some("Token2".to_string()),
                symbol: Some("Token2".to_string()),
                decimals: Some(BigInt(6.to_string())),
            },
            balance: BigInt("123".to_string()),
            vault_id: BigInt(B256::random().to_string()),
            owner: Bytes(local_evm.anvil.addresses()[0].to_string()),
            orderbook: Orderbook {
                id: Bytes(orderbook.address().to_string()),
            },
            orders_as_input: vec![],
            orders_as_output: vec![],
            balance_changes: vec![],
        }];

        let outputs = vec![Vault {
            id: Bytes(B256::random().to_string()),
            token: Erc20 {
                id: Bytes(token1.address().to_string()),
                address: Bytes(token1.address().to_string()),
                name: Some("Token1".to_string()),
                symbol: Some("token1".to_string()),
                decimals: Some(BigInt(18.to_string())),
            },
            balance: BigInt("123".to_string()),
            vault_id: BigInt(B256::random().to_string()),
            owner: Bytes(local_evm.anvil.addresses()[0].to_string()),
            orderbook: Orderbook {
                id: Bytes(orderbook.address().to_string()),
            },
            orders_as_input: vec![],
            orders_as_output: vec![],
            balance_changes: vec![],
        }];

        let order = Order {
            id: Bytes(B256::random().to_string()),
            orderbook: Orderbook {
                id: Bytes(orderbook.address().to_string()),
            },
            order_bytes: Bytes(order),
            order_hash: Bytes(B256::random().to_string()),
            owner: Bytes(local_evm.anvil.addresses()[0].to_string()),
            outputs,
            inputs,
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: BigInt(0.to_string()),
            trades: vec![]
        };

        let rpc_url = local_evm.url();

        let result = batch_order_quotes(vec![order], None, rpc_url)
            .await
            .unwrap();

        let quote = result[0].data.unwrap();

        assert_eq!(quote.max_output, parse_ether("16").unwrap());
        assert_eq!(quote.ratio, parse_ether("52").unwrap());
    }
}
