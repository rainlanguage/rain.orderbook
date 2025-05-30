use crate::{
    error::Error,
    quote::{BatchQuoteTarget, QuoteTarget},
    OrderQuoteValue,
};
use alloy::primitives::{Address, U256};
use alloy_ethers_typecast::transaction::ReadableClient;
use rain_orderbook_bindings::IOrderBookV4::{OrderV3, Quote};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderQuotesResponse {
    pub pair: Pair,
    pub block_number: u64,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub error: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BatchOrderQuotesResponse);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub pair_name: String,
    pub input_index: u32,
    pub output_index: u32,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(Pair);

pub async fn get_order_quotes(
    orders: Vec<SgOrder>,
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
            ReadableClient::new_from_urls(vec![rpc_url.clone()])?
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
                            block_number: req_block_number,
                            success: true,
                            data: Some(quote_value),
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(BatchOrderQuotesResponse {
                            pair,
                            block_number: req_block_number,
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
                    block_number: req_block_number,
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
        hex::{encode_prefixed, FromHexError},
        primitives::B256,
        providers::Provider,
        sol_types::{SolCall, SolValue},
    };
    use alloy_ethers_typecast::transaction::ReadableClientError;
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_subgraph_client::types::{
        common::{SgBigInt, SgBytes, SgErc20, SgOrderbook, SgVault},
        order_detail_traits::OrderDetailError,
    };
    use rain_orderbook_test_fixtures::LocalEvm;

    struct TestSetup {
        local_evm: LocalEvm,
        owner: Address,
        token1: SgErc20,
        token2: SgErc20,
        orderbook: Address,
    }

    async fn setup_test() -> TestSetup {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();

        let token1 = local_evm
            .deploy_new_token("Token1", "Token1", 18, U256::MAX, owner)
            .await;
        let token2 = local_evm
            .deploy_new_token("Token2", "Token2", 18, U256::MAX, owner)
            .await;
        let orderbook = *local_evm.orderbook.address();

        TestSetup {
            local_evm,
            owner,
            token1: SgErc20 {
                id: SgBytes(token1.address().to_string()),
                address: SgBytes(token1.address().to_string()),
                name: Some("Token1".to_string()),
                symbol: Some("Token1".to_string()),
                decimals: Some(SgBigInt(18.to_string())),
            },
            token2: SgErc20 {
                id: SgBytes(token2.address().to_string()),
                address: SgBytes(token2.address().to_string()),
                name: Some("Token2".to_string()),
                symbol: Some("Token2".to_string()),
                decimals: Some(SgBigInt(18.to_string())),
            },
            orderbook,
        }
    }

    fn create_dotrain_config(setup: &TestSetup) -> String {
        format!(
            r#"
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
        deployer: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
/* use io addresses in context as calculate-io maxoutput and ratio */
amount price: context<3 0>() context<4 0>();
#handle-add-order
:;
#handle-io
:;
"#,
            rpc_url = setup.local_evm.url(),
            orderbook = setup.orderbook,
            deployer = setup.local_evm.deployer.address(),
            token1 = setup.token1.address.0,
            token2 = setup.token2.address.0,
        )
    }

    async fn create_order(setup: &TestSetup, dotrain: String) -> String {
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
            .try_into_call(vec![setup.local_evm.url()])
            .await
            .unwrap()
            .abi_encode();

        encode_prefixed(
            setup
                .local_evm
                .add_order(&calldata, setup.owner)
                .await
                .0
                .order
                .abi_encode(),
        )
    }

    fn create_vault(setup: &TestSetup, token: &SgErc20) -> SgVault {
        SgVault {
            id: SgBytes(B256::random().to_string()),
            token: token.clone(),
            balance: SgBigInt("123".to_string()),
            vault_id: SgBigInt(B256::random().to_string()),
            owner: SgBytes(setup.local_evm.anvil.addresses()[0].to_string()),
            orderbook: SgOrderbook {
                id: SgBytes(setup.orderbook.to_string()),
            },
            orders_as_input: vec![],
            orders_as_output: vec![],
            balance_changes: vec![],
        }
    }

    fn create_sg_order(
        setup: &TestSetup,
        order_bytes: String,
        inputs: Vec<SgVault>,
        outputs: Vec<SgVault>,
    ) -> SgOrder {
        SgOrder {
            id: SgBytes(B256::random().to_string()),
            orderbook: SgOrderbook {
                id: SgBytes(setup.orderbook.to_string()),
            },
            order_bytes: SgBytes(order_bytes),
            order_hash: SgBytes(B256::random().to_string()),
            owner: SgBytes(setup.local_evm.anvil.addresses()[0].to_string()),
            outputs,
            inputs,
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: SgBigInt(0.to_string()),
            trades: vec![],
            remove_events: vec![],
        }
    }

    #[tokio::test]
    async fn test_get_order_quotes_ok() {
        let setup = setup_test().await;

        // Deposit in token1 and token2 vaults
        setup
            .local_evm
            .deposit(
                setup.owner,
                Address::from_str(&setup.token1.address.0).unwrap(),
                U256::MAX,
                U256::from(1),
            )
            .await;
        setup
            .local_evm
            .deposit(
                setup.owner,
                Address::from_str(&setup.token2.address.0).unwrap(),
                U256::MAX,
                U256::from(1),
            )
            .await;

        let dotrain = create_dotrain_config(&setup);
        let order = create_order(&setup, dotrain).await;

        let vault1 = create_vault(&setup, &setup.token1);
        let vault2 = create_vault(&setup, &setup.token2);

        // does not follow the actual original order's io order
        let inputs = vec![vault2.clone(), vault1.clone()];
        let outputs = vec![vault2.clone(), vault1.clone()];

        let order = create_sg_order(&setup, order, inputs, outputs);

        let result = get_order_quotes(vec![order], None, setup.local_evm.url(), None)
            .await
            .unwrap();

        let token1_as_u256 = U256::from_str(&setup.token1.address.0).unwrap();
        let token2_as_u256 = U256::from_str(&setup.token2.address.0).unwrap();
        let block_number = setup.local_evm.provider.get_block_number().await.unwrap();
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

    #[tokio::test]
    async fn test_get_order_quotes_err() {
        let setup = setup_test().await;
        let dotrain = create_dotrain_config(&setup);
        let order = create_order(&setup, dotrain).await;

        // Test invalid orderbook address
        let mut invalid_order = create_sg_order(&setup, order.clone(), vec![], vec![]);
        invalid_order.orderbook.id = SgBytes("invalid_address".to_string());

        let err = get_order_quotes(vec![invalid_order], None, setup.local_evm.url(), None)
            .await
            .unwrap_err();

        assert!(matches!(err, Error::FromHexError(FromHexError::OddLength)));

        // Test invalid order bytes
        let invalid_order = create_sg_order(&setup, B256::random().to_string(), vec![], vec![]);

        let err = get_order_quotes(vec![invalid_order], None, setup.local_evm.url(), None)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::OrderDetailError(OrderDetailError::AbiDecode(_))
        ));

        // Test invalid RPC URL
        let valid_order = create_sg_order(&setup, order, vec![], vec![]);

        let err = get_order_quotes(vec![valid_order], None, "invalid_rpc_url".to_string(), None)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            Error::RpcCallError(ReadableClientError::CreateReadableClientHttpError(_))
        ));
    }
}
