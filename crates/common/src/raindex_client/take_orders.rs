use super::*;
use crate::raindex_client::order_quotes::RaindexOrderQuote;
use crate::raindex_client::orders::RaindexOrder;
use alloy::primitives::Address;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// A candidate for taking an order, representing a specific IO pair direction
/// with its quote data.
///
/// `TakeOrderCandidate` represents a "thing we could include in
/// `TakeOrdersConfigV4.orders[]`". It contains the decoded `OrderV4` along
/// with the specific input/output indices and the quote data (max_output and ratio).
#[derive(Clone, Debug)]
pub struct TakeOrderCandidate {
    /// The decoded OrderV4 from on-chain order bytes
    pub order: OrderV4,
    /// Index into the order's validInputs array
    pub input_io_index: u32,
    /// Index into the order's validOutputs array
    pub output_io_index: u32,
    /// Maximum output amount in output token units (from quote2.outputMax)
    pub max_output: Float,
    /// Ratio: input per 1 output, from the order's perspective (from quote2.IORatio)
    pub ratio: Float,
}

/// Builds a list of `TakeOrderCandidate`s from a list of `RaindexOrder`s for a
/// specific trading direction.
///
/// This function:
/// 1. Calls `RaindexOrder::get_quotes` on each order
/// 2. Filters quotes to the specified `(input_token, output_token)` direction
/// 3. Filters to non-zero capacity (`max_output > 0`)
/// 4. Returns candidates that can be used to build `TakeOrdersConfigV4`
///
/// # Arguments
/// * `orders` - List of orders to quote
/// * `input_token` - The taker's input token (what the taker gives, order receives)
/// * `output_token` - The taker's output token (what the taker receives, order gives)
/// * `block_number` - Optional block number for historical quotes
/// * `gas` - Optional gas limit for quote simulations
///
/// # Returns
/// A vector of `TakeOrderCandidate`s for the specified direction with non-zero capacity.
pub async fn build_take_order_candidates_for_pair(
    orders: &[RaindexOrder],
    input_token: Address,
    output_token: Address,
    block_number: Option<u64>,
    gas: Option<u64>,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    let mut candidates: Vec<TakeOrderCandidate> = Vec::new();
    let gas_string = gas.map(|g| g.to_string());

    for order in orders {
        let quotes = order.get_quotes(block_number, gas_string.clone()).await?;

        let order_v4: OrderV4 = order.try_into()?;

        for quote in quotes {
            if let Some(candidate) =
                try_build_candidate(&order_v4, &quote, input_token, output_token)?
            {
                candidates.push(candidate);
            }
        }
    }

    Ok(candidates)
}

/// Try to build a TakeOrderCandidate from a quote if it matches the direction
/// and has non-zero capacity.
fn try_build_candidate(
    order: &OrderV4,
    quote: &RaindexOrderQuote,
    input_token: Address,
    output_token: Address,
) -> Result<Option<TakeOrderCandidate>, RaindexError> {
    if !quote.success {
        return Ok(None);
    }

    let data = match &quote.data {
        Some(d) => d,
        None => return Ok(None),
    };

    let input_io_index = quote.pair.input_index;
    let output_io_index = quote.pair.output_index;

    if input_io_index as usize >= order.validInputs.len() {
        return Ok(None);
    }
    if output_io_index as usize >= order.validOutputs.len() {
        return Ok(None);
    }

    let order_input_token = order.validInputs[input_io_index as usize].token;
    let order_output_token = order.validOutputs[output_io_index as usize].token;

    if order_input_token != input_token || order_output_token != output_token {
        return Ok(None);
    }

    if data.max_output.lte(Float::zero()?)? {
        return Ok(None);
    }

    Ok(Some(TakeOrderCandidate {
        order: order.clone(),
        input_io_index,
        output_io_index,
        max_output: data.max_output,
        ratio: data.ratio,
    }))
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::super::*;
        use alloy::primitives::{Address, U256};
        use rain_math_float::Float;
        use rain_orderbook_subgraph_client::utils::float::{F1, F2};

        #[test]
        fn test_try_build_candidate_wrong_direction() {
            use crate::raindex_client::order_quotes::{RaindexOrderQuote, RaindexOrderQuoteValue};
            use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
            use rain_orderbook_quote::Pair;

            let token_a = Address::from([4u8; 20]);
            let token_b = Address::from([5u8; 20]);

            let order = OrderV4 {
                owner: Address::from([1u8; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([2u8; 20]),
                    store: Address::from([3u8; 20]),
                    bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
                },
                validInputs: vec![IOV2 {
                    token: token_a,
                    vaultId: U256::from(100).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: token_b,
                    vaultId: U256::from(200).into(),
                }],
            };

            let quote = RaindexOrderQuote {
                pair: Pair {
                    pair_name: "A/B".to_string(),
                    input_index: 0,
                    output_index: 0,
                },
                block_number: 1,
                data: Some(RaindexOrderQuoteValue {
                    max_output: F1,
                    formatted_max_output: "1".to_string(),
                    max_input: F1,
                    formatted_max_input: "1".to_string(),
                    ratio: F1,
                    formatted_ratio: "1".to_string(),
                    inverse_ratio: F1,
                    formatted_inverse_ratio: "1".to_string(),
                }),
                success: true,
                error: None,
            };

            // Try with reversed direction - should return None
            let result = try_build_candidate(
                &order, &quote, token_b, // reversed
                token_a, // reversed
            )
            .unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn test_try_build_candidate_zero_capacity() {
            use crate::raindex_client::order_quotes::{RaindexOrderQuote, RaindexOrderQuoteValue};
            use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
            use rain_orderbook_quote::Pair;

            let token_a = Address::from([4u8; 20]);
            let token_b = Address::from([5u8; 20]);

            let order = OrderV4 {
                owner: Address::from([1u8; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([2u8; 20]),
                    store: Address::from([3u8; 20]),
                    bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
                },
                validInputs: vec![IOV2 {
                    token: token_a,
                    vaultId: U256::from(100).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: token_b,
                    vaultId: U256::from(200).into(),
                }],
            };

            let quote = RaindexOrderQuote {
                pair: Pair {
                    pair_name: "A/B".to_string(),
                    input_index: 0,
                    output_index: 0,
                },
                block_number: 1,
                data: Some(RaindexOrderQuoteValue {
                    max_output: Float::zero().unwrap(), // zero capacity
                    formatted_max_output: "0".to_string(),
                    max_input: Float::zero().unwrap(),
                    formatted_max_input: "0".to_string(),
                    ratio: F1,
                    formatted_ratio: "1".to_string(),
                    inverse_ratio: F1,
                    formatted_inverse_ratio: "1".to_string(),
                }),
                success: true,
                error: None,
            };

            // Should return None due to zero capacity
            let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn test_try_build_candidate_success() {
            use crate::raindex_client::order_quotes::{RaindexOrderQuote, RaindexOrderQuoteValue};
            use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
            use rain_orderbook_quote::Pair;

            let token_a = Address::from([4u8; 20]);
            let token_b = Address::from([5u8; 20]);

            let order = OrderV4 {
                owner: Address::from([1u8; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([2u8; 20]),
                    store: Address::from([3u8; 20]),
                    bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
                },
                validInputs: vec![IOV2 {
                    token: token_a,
                    vaultId: U256::from(100).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: token_b,
                    vaultId: U256::from(200).into(),
                }],
            };

            let quote = RaindexOrderQuote {
                pair: Pair {
                    pair_name: "A/B".to_string(),
                    input_index: 0,
                    output_index: 0,
                },
                block_number: 1,
                data: Some(RaindexOrderQuoteValue {
                    max_output: F2,
                    formatted_max_output: "2".to_string(),
                    max_input: F1,
                    formatted_max_input: "1".to_string(),
                    ratio: F1,
                    formatted_ratio: "1".to_string(),
                    inverse_ratio: F1,
                    formatted_inverse_ratio: "1".to_string(),
                }),
                success: true,
                error: None,
            };

            // Correct direction, non-zero capacity - should succeed
            let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

            assert!(result.is_some());
            let candidate = result.unwrap();
            assert_eq!(candidate.input_io_index, 0);
            assert_eq!(candidate.output_io_index, 0);
            assert!(candidate.max_output.eq(F2).unwrap());
        }

        #[test]
        fn test_try_build_candidate_failed_quote() {
            use crate::raindex_client::order_quotes::RaindexOrderQuote;
            use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
            use rain_orderbook_quote::Pair;

            let token_a = Address::from([4u8; 20]);
            let token_b = Address::from([5u8; 20]);

            let order = OrderV4 {
                owner: Address::from([1u8; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([2u8; 20]),
                    store: Address::from([3u8; 20]),
                    bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
                },
                validInputs: vec![IOV2 {
                    token: token_a,
                    vaultId: U256::from(100).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: token_b,
                    vaultId: U256::from(200).into(),
                }],
            };

            let quote = RaindexOrderQuote {
                pair: Pair {
                    pair_name: "A/B".to_string(),
                    input_index: 0,
                    output_index: 0,
                },
                block_number: 1,
                data: None,
                success: false, // failed quote
                error: Some("Quote failed".to_string()),
            };

            // Failed quote should return None
            let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

            assert!(result.is_none());
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod integration_tests {
        use super::super::*;
        use crate::add_order::AddOrderArgs;
        use crate::dotrain_order::DotrainOrder;
        use crate::raindex_client::orders::RaindexOrder;
        use crate::raindex_client::RaindexClient;
        use alloy::hex::encode_prefixed;
        use alloy::primitives::{B256, U256};
        use alloy::sol_types::{SolCall, SolValue};
        use rain_orderbook_app_settings::spec_version::SpecVersion;
        use rain_orderbook_subgraph_client::types::common::{
            SgBigInt, SgBytes, SgErc20, SgOrder, SgOrderbook, SgVault,
        };
        use rain_orderbook_subgraph_client::utils::float::F6;
        use rain_orderbook_test_fixtures::LocalEvm;
        use std::rc::Rc;

        struct TestSetup {
            local_evm: LocalEvm,
            owner: Address,
            token1: Address,
            token2: Address,
            token1_sg: SgErc20,
            token2_sg: SgErc20,
            orderbook: Address,
            raindex_client: Rc<RaindexClient>,
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

            let yaml = format!(
                r#"
version: {spec_version}
networks:
    test-network:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
subgraphs:
    test-sg: http://localhost:0/notused
metaboards:
    test-mb: http://localhost:0/notused
deployers:
    test-deployer:
        address: {deployer}
        network: test-network
orderbooks:
    test-orderbook:
        address: {orderbook}
        network: test-network
        subgraph: test-sg
        local-db-remote: remote
        deployment-block: 0
tokens:
    token1:
        network: test-network
        address: {token1}
        decimals: 18
        label: Token1
        symbol: TKN1
    token2:
        network: test-network
        address: {token2}
        decimals: 18
        label: Token2
        symbol: TKN2
"#,
                spec_version = SpecVersion::current(),
                rpc_url = local_evm.url(),
                orderbook = orderbook,
                deployer = local_evm.deployer.address(),
                token1 = token1.address(),
                token2 = token2.address(),
            );

            let raindex_client = Rc::new(RaindexClient::new(vec![yaml], None).expect("Valid yaml"));

            TestSetup {
                token1: *token1.address(),
                token2: *token2.address(),
                token1_sg: SgErc20 {
                    id: SgBytes(token1.address().to_string()),
                    address: SgBytes(token1.address().to_string()),
                    name: Some("Token1".to_string()),
                    symbol: Some("Token1".to_string()),
                    decimals: Some(SgBigInt(18.to_string())),
                },
                token2_sg: SgErc20 {
                    id: SgBytes(token2.address().to_string()),
                    address: SgBytes(token2.address().to_string()),
                    name: Some("Token2".to_string()),
                    symbol: Some("Token2".to_string()),
                    decimals: Some(SgBigInt(18.to_string())),
                },
                local_evm,
                owner,
                orderbook,
                raindex_client,
            }
        }

        fn create_dotrain_config(setup: &TestSetup) -> String {
            format!(
                r#"
version: {spec_version}
networks:
    test-network:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
deployers:
    test-deployer:
        network: test-network
        address: {deployer}
tokens:
    t1:
        network: test-network
        address: {token1}
        decimals: 18
        label: Token1
        symbol: Token1
    t2:
        network: test-network
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
orderbook:
    test-orderbook:
        address: {orderbook}
orders:
    test-order:
        inputs:
            - token: t1
            - token: t2
        outputs:
            - token: t1
              vault-id: 0x01
            - token: t2
              vault-id: 0x01
scenarios:
    test-scenario:
        deployer: test-deployer
        bindings:
            max-amount: 1000
deployments:
    test-deployment:
        scenario: test-scenario
        order: test-order
---
#max-amount !Max output amount
#calculate-io
/* Return fixed max_output and ratio */
amount price: 100 2;
#handle-add-order
:;
#handle-io
:;
"#,
                rpc_url = setup.local_evm.url(),
                orderbook = setup.orderbook,
                deployer = setup.local_evm.deployer.address(),
                token1 = setup.token1,
                token2 = setup.token2,
                spec_version = SpecVersion::current(),
            )
        }

        async fn deploy_order(setup: &TestSetup, dotrain: String) -> (String, B256) {
            let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
            let deployment = dotrain_order
                .dotrain_yaml()
                .get_deployment("test-deployment")
                .unwrap();
            let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
                .await
                .unwrap()
                .try_into_call(vec![setup.local_evm.url()])
                .await
                .unwrap()
                .abi_encode();

            let (event, _) = setup.local_evm.add_order(&calldata, setup.owner).await;
            let order_bytes = encode_prefixed(event.order.abi_encode());
            let order_hash = B256::from(event.orderHash);
            (order_bytes, order_hash)
        }

        fn create_vault(vault_id: B256, setup: &TestSetup, token: &SgErc20) -> SgVault {
            SgVault {
                id: SgBytes(vault_id.to_string()),
                token: token.clone(),
                balance: SgBytes(F6.as_hex()),
                vault_id: SgBytes(vault_id.to_string()),
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
            order_hash: B256,
            inputs: Vec<SgVault>,
            outputs: Vec<SgVault>,
        ) -> SgOrder {
            SgOrder {
                id: SgBytes(order_hash.to_string()),
                orderbook: SgOrderbook {
                    id: SgBytes(setup.orderbook.to_string()),
                },
                order_bytes: SgBytes(order_bytes),
                order_hash: SgBytes(order_hash.to_string()),
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
        async fn test_build_take_order_candidates_for_correct_direction() {
            let setup = setup_test().await;

            let vault_id = B256::from(U256::from(1u64));

            // Deposit tokens so the order has capacity
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token1,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token2,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;

            let dotrain = create_dotrain_config(&setup);
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let inputs = vec![vault1.clone(), vault2.clone()];
            let outputs = vec![vault1.clone(), vault2.clone()];

            let sg_order = create_sg_order(&setup, order_bytes, order_hash, inputs, outputs);

            // Create RaindexOrder from SgOrder
            let raindex_order = RaindexOrder::try_from_sg_order(
                Rc::clone(&setup.raindex_client),
                123, // chain_id
                sg_order,
                None,
            )
            .expect("Should create RaindexOrder");

            // Test direction: token1 as input, token2 as output
            // This means we (the taker) give token1 and receive token2
            let candidates = build_take_order_candidates_for_pair(
                std::slice::from_ref(&raindex_order),
                setup.token1, // input (we give)
                setup.token2, // output (we receive)
                None,
                None,
            )
            .await
            .expect("Should build candidates");

            // Should have exactly one candidate for this direction
            assert_eq!(
                candidates.len(),
                1,
                "Expected 1 candidate for token1->token2 direction"
            );

            let candidate = &candidates[0];
            assert!(
                candidate.max_output.gt(Float::zero().unwrap()).unwrap(),
                "max_output should be > 0"
            );
        }

        #[tokio::test]
        async fn test_build_take_order_candidates_filters_wrong_direction() {
            let setup = setup_test().await;

            let vault_id = B256::from(U256::from(1u64));

            // Deposit tokens
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token1,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token2,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;

            let dotrain = create_dotrain_config(&setup);
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let inputs = vec![vault1.clone(), vault2.clone()];
            let outputs = vec![vault1.clone(), vault2.clone()];

            let sg_order = create_sg_order(&setup, order_bytes, order_hash, inputs, outputs);

            let raindex_order = RaindexOrder::try_from_sg_order(
                Rc::clone(&setup.raindex_client),
                123,
                sg_order,
                None,
            )
            .expect("Should create RaindexOrder");

            // Query for a token that doesn't exist in the order
            let fake_token = Address::from([0xABu8; 20]);
            let candidates = build_take_order_candidates_for_pair(
                &[raindex_order],
                fake_token, // not in order
                setup.token2,
                None,
                None,
            )
            .await
            .expect("Should not fail, just return empty");

            // No candidates should match
            assert_eq!(
                candidates.len(),
                0,
                "Expected 0 candidates for non-existent token direction"
            );
        }

        #[tokio::test]
        async fn test_build_take_order_candidates_filters_zero_capacity() {
            let setup = setup_test().await;

            // Create order but don't deposit anything - zero capacity
            let dotrain = create_dotrain_config(&setup);
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let vault_id = B256::from(U256::from(1u64));
            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let inputs = vec![vault1.clone(), vault2.clone()];
            let outputs = vec![vault1.clone(), vault2.clone()];

            let sg_order = create_sg_order(&setup, order_bytes, order_hash, inputs, outputs);

            let raindex_order = RaindexOrder::try_from_sg_order(
                Rc::clone(&setup.raindex_client),
                123,
                sg_order,
                None,
            )
            .expect("Should create RaindexOrder");

            // Even with correct direction, should return empty due to zero capacity
            // (no deposits were made)
            let candidates = build_take_order_candidates_for_pair(
                &[raindex_order],
                setup.token1,
                setup.token2,
                None,
                None,
            )
            .await
            .expect("Should not fail, just filter out zero capacity");

            // Order has zero capacity so should be filtered out
            assert_eq!(
                candidates.len(),
                0,
                "Expected 0 candidates for zero-capacity order"
            );
        }

        #[tokio::test]
        async fn test_build_take_order_candidates_multiple_orders_mixed() {
            let setup = setup_test().await;

            let vault_id = B256::from(U256::from(1u64));

            // Deposit for first order only
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token1,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;
            setup
                .local_evm
                .deposit(
                    setup.owner,
                    setup.token2,
                    U256::from(10).pow(U256::from(20)),
                    18,
                    vault_id,
                )
                .await;

            let dotrain = create_dotrain_config(&setup);
            let (order_bytes1, order_hash1) = deploy_order(&setup, dotrain.clone()).await;
            let (order_bytes2, order_hash2) = deploy_order(&setup, dotrain).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let inputs = vec![vault1.clone(), vault2.clone()];
            let outputs = vec![vault1.clone(), vault2.clone()];

            let sg_order1 = create_sg_order(
                &setup,
                order_bytes1,
                order_hash1,
                inputs.clone(),
                outputs.clone(),
            );
            let sg_order2 = create_sg_order(&setup, order_bytes2, order_hash2, inputs, outputs);

            let raindex_order1 = RaindexOrder::try_from_sg_order(
                Rc::clone(&setup.raindex_client),
                123,
                sg_order1,
                None,
            )
            .expect("Should create RaindexOrder");

            let raindex_order2 = RaindexOrder::try_from_sg_order(
                Rc::clone(&setup.raindex_client),
                123,
                sg_order2,
                None,
            )
            .expect("Should create RaindexOrder");

            // Both orders should have capacity (same vault with deposited tokens)
            let candidates = build_take_order_candidates_for_pair(
                &[raindex_order1, raindex_order2],
                setup.token1,
                setup.token2,
                None,
                None,
            )
            .await
            .expect("Should build candidates");

            // Both orders should produce candidates for token1->token2 direction
            assert_eq!(
                candidates.len(),
                2,
                "Expected 2 candidates from 2 orders with capacity"
            );
        }
    }
}
