use crate::{
    error::Error,
    quote::{BatchQuoteTarget, QuoteTarget},
    OrderQuoteValue,
};
use alloy::primitives::{Address, U256};
use alloy_ethers_typecast::transaction::ReadableClient;
use rain_orderbook_bindings::IOrderBookV4::{OrderV3, Quote};
use rain_orderbook_subgraph_client::types::common::Order;
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

pub async fn get_order_quotes(
    orders: Vec<Order>,
    block_number: Option<u64>,
    rpc_url: String,
    gas: Option<U256>,
) -> Result<Vec<BatchOrderQuotesResponse>, Error> {
    let mut results: Vec<BatchOrderQuotesResponse> = Vec::new();

    for order in &orders {
        let mut pairs: Vec<Pair> = Vec::new();
        let mut quote_targets: Vec<QuoteTarget> = Vec::new();
        let order_struct: OrderV3 = order.clone().try_into()?;
        let orderbook = Address::from_str(&order.orderbook.id.0)?;

        for (input_index, input) in order_struct.validInputs.iter().enumerate() {
            for (output_index, output) in order_struct.validOutputs.iter().enumerate() {
                let pair_name = format!(
                    "{}/{}",
                    order
                        .inputs
                        .iter()
                        .find_map(|v| {
                            Address::from_str(&v.token.address.0).ok().and_then(|add| {
                                add.eq(&input.token).then_some(
                                    v.token.symbol.clone().unwrap_or("UNKNOWN".to_string()),
                                )
                            })
                        })
                        .unwrap_or("UNKNOWN".to_string()),
                    order
                        .outputs
                        .iter()
                        .find_map(|v| {
                            Address::from_str(&v.token.address.0).ok().and_then(|add| {
                                add.eq(&output.token).then_some(
                                    v.token.symbol.clone().unwrap_or("UNKNOWN".to_string()),
                                )
                            })
                        })
                        .unwrap_or("UNKNOWN".to_string())
                );

                let quote_target = QuoteTarget {
                    orderbook,
                    quote_config: Quote {
                        order: order_struct.clone(),
                        inputIOIndex: U256::from(input_index),
                        outputIOIndex: U256::from(output_index),
                        signedContext: vec![],
                    },
                };

                if input.token != output.token {
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
            .do_quote(&rpc_url, Some(req_block_number), gas, None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::B256,
        providers::Provider,
        sol_types::{SolCall, SolValue},
    };
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_subgraph_client::types::common::{BigInt, Bytes, Erc20, Orderbook, Vault};
    use rain_orderbook_test_fixtures::LocalEvm;

    #[tokio::test]
    async fn test_get_order_quotes() {
        let mut local_evm = LocalEvm::new().await;

        let owner = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm
            .deploy_new_token("Token1", "Token1", 18, U256::MAX, owner)
            .await;
        let token2 = local_evm
            .deploy_new_token("Token2", "Token2", 18, U256::MAX, owner)
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
        symbol: token1
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: t1
            - token: t2
        outputs:
            - token: t1
              vault-id: 0x01
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
/* use io addresses in context as calculate-io maxoutput and ratio */
amount price: context<3 0>() context<4 0>();
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

        // add order
        let order = encode_prefixed(
            local_evm
                .add_order(&calldata, owner)
                .await
                .0
                .order
                .abi_encode(),
        );
        // deposit in token1 and token2 vaults
        // deposit MAX so we can get token addresses as the quote result
        local_evm
            .deposit(owner, *token1.address(), U256::MAX, U256::from(1))
            .await;
        local_evm
            .deposit(owner, *token2.address(), U256::MAX, U256::from(1))
            .await;

        let vault1 = Vault {
            id: Bytes(B256::random().to_string()),
            token: Erc20 {
                id: Bytes(token1.address().to_string()),
                address: Bytes(token1.address().to_string()),
                name: Some("Token1".to_string()),
                symbol: Some("Token1".to_string()),
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
        };
        let vault2 = Vault {
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
        };

        // does not follow the actual original order's io order
        let inputs = vec![vault2.clone(), vault1.clone()];
        let outputs = vec![vault2.clone(), vault1.clone()];

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
            trades: vec![],
        };

        let result = get_order_quotes(vec![order], None, local_evm.url(), None)
            .await
            .unwrap();

        let token1_as_u256 = U256::from_str(&token1.address().to_string()).unwrap();
        let token2_as_u256 = U256::from_str(&token2.address().to_string()).unwrap();
        let block_number = U256::from(local_evm.provider.get_block_number().await.unwrap());
        let expected = vec![
            BatchOrderQuotesResponse {
                pair: Pair {
                    pair_name: "Token1/Token2".to_string(),
                    input_index: 0,
                    output_index: 1,
                },
                block_number,
                data: Some(OrderQuoteValue {
                    max_output: token1_as_u256,
                    ratio: token2_as_u256,
                }),
                success: true,
                error: None,
            },
            BatchOrderQuotesResponse {
                pair: Pair {
                    pair_name: "Token2/Token1".to_string(),
                    input_index: 1,
                    output_index: 0,
                },
                block_number,
                data: Some(OrderQuoteValue {
                    max_output: token2_as_u256,
                    ratio: token1_as_u256,
                }),
                success: true,
                error: None,
            },
        ];
        assert_eq!(result, expected);
    }
}
