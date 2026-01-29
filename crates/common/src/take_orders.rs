use crate::raindex_client::order_quotes::RaindexOrderQuote;
use crate::raindex_client::orders::RaindexOrder;
use crate::raindex_client::RaindexError;
use alloy::primitives::Address;
use futures::StreamExt;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

const DEFAULT_QUOTE_CONCURRENCY: usize = 5;

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
    let gas_string = gas.map(|g| g.to_string());

    let quote_results: Vec<Result<_, RaindexError>> =
        futures::stream::iter(orders.iter().map(|order| {
            let gas_string = gas_string.clone();
            async move { order.get_quotes(block_number, gas_string).await }
        }))
        .buffered(DEFAULT_QUOTE_CONCURRENCY)
        .collect()
        .await;

    let mut candidates: Vec<TakeOrderCandidate> = Vec::new();

    for (order, quotes_result) in orders.iter().zip(quote_results) {
        let quotes = quotes_result?;
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
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::add_order::AddOrderArgs;
    use crate::dotrain_order::DotrainOrder;
    use crate::raindex_client::order_quotes::{RaindexOrderQuote, RaindexOrderQuoteValue};
    use crate::raindex_client::orders::RaindexOrder;
    use crate::raindex_client::RaindexClient;
    use alloy::hex::encode_prefixed;
    use alloy::primitives::{Address, B256, U256};
    use alloy::sol_types::{SolCall, SolValue};
    use rain_math_float::Float;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
    use rain_orderbook_quote::Pair;
    use rain_orderbook_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrder, SgOrderbook, SgVault,
    };
    use rain_orderbook_subgraph_client::utils::float::{F1, F2, F6};
    use rain_orderbook_test_fixtures::LocalEvm;
    use std::rc::Rc;

    fn make_basic_order(input_token: Address, output_token: Address) -> OrderV4 {
        OrderV4 {
            owner: Address::from([1u8; 20]),
            nonce: U256::from(1).into(),
            evaluable: EvaluableV4 {
                interpreter: Address::from([2u8; 20]),
                store: Address::from([3u8; 20]),
                bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
            },
            validInputs: vec![IOV2 {
                token: input_token,
                vaultId: U256::from(100).into(),
            }],
            validOutputs: vec![IOV2 {
                token: output_token,
                vaultId: U256::from(200).into(),
            }],
        }
    }

    fn make_quote_value(
        max_output: Float,
        max_input: Float,
        ratio: Float,
    ) -> RaindexOrderQuoteValue {
        RaindexOrderQuoteValue {
            max_output,
            formatted_max_output: max_output.format().unwrap(),
            max_input,
            formatted_max_input: max_input.format().unwrap(),
            ratio,
            formatted_ratio: ratio.format().unwrap(),
            inverse_ratio: ratio,
            formatted_inverse_ratio: ratio.format().unwrap(),
        }
    }

    fn make_quote(
        input_index: u32,
        output_index: u32,
        data: Option<RaindexOrderQuoteValue>,
        success: bool,
    ) -> RaindexOrderQuote {
        RaindexOrderQuote {
            pair: Pair {
                pair_name: "A/B".to_string(),
                input_index,
                output_index,
            },
            block_number: 1,
            data,
            success,
            error: if success {
                None
            } else {
                Some("Quote failed".to_string())
            },
        }
    }

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

    fn create_dotrain_config_with_vault_id(setup: &TestSetup, vault_id: &str) -> String {
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
              vault-id: {vault_id}
            - token: t2
              vault-id: {vault_id}
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
            vault_id = vault_id,
        )
    }

    fn create_dotrain_config(setup: &TestSetup) -> String {
        create_dotrain_config_with_vault_id(setup, "0x01")
    }

    async fn deploy_order(setup: &TestSetup, dotrain: String) -> (String, B256) {
        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("test-deployment")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment, None)
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

    fn standard_deposit_amount() -> U256 {
        U256::from(10).pow(U256::from(20))
    }

    async fn fund_standard_two_token_vault(setup: &TestSetup, vault_id: B256) {
        let amount = standard_deposit_amount();
        setup
            .local_evm
            .deposit(setup.owner, setup.token1, amount, 18, vault_id)
            .await;
        setup
            .local_evm
            .deposit(setup.owner, setup.token2, amount, 18, vault_id)
            .await;
    }

    async fn make_raindex_order(
        setup: &TestSetup,
        dotrain: String,
        inputs: Vec<SgVault>,
        outputs: Vec<SgVault>,
    ) -> RaindexOrder {
        let (order_bytes, order_hash) = deploy_order(setup, dotrain).await;
        let sg_order = create_sg_order(setup, order_bytes, order_hash, inputs, outputs);
        RaindexOrder::try_from_sg_order(Rc::clone(&setup.raindex_client), 123, sg_order, None)
            .expect("Should create RaindexOrder")
    }

    #[test]
    fn test_try_build_candidate_wrong_direction() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, Some(make_quote_value(F1, F1, F1)), true);

        let result = try_build_candidate(&order, &quote, token_b, token_a).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_zero_capacity() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);
        let zero = Float::zero().unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(zero, zero, F1)), true);

        let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_success() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, Some(make_quote_value(F2, F1, F1)), true);

        let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.input_io_index, 0);
        assert_eq!(candidate.output_io_index, 0);
        assert!(candidate.max_output.eq(F2).unwrap());
    }

    #[test]
    fn test_try_build_candidate_failed_quote() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, None, false);

        let result = try_build_candidate(&order, &quote, token_a, token_b);

        assert!(
            result.is_ok(),
            "Failed quote must not cause an error, got: {:?}",
            result.unwrap_err()
        );
        assert!(
            result.unwrap().is_none(),
            "Failed quote must not produce a candidate"
        );
    }

    #[test]
    fn test_try_build_candidate_out_of_bounds_indices() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);

        let quote_bad_input_index = make_quote(99, 0, Some(make_quote_value(F1, F1, F1)), true);
        let result = try_build_candidate(&order, &quote_bad_input_index, token_a, token_b);
        assert!(
            result.is_ok(),
            "Out-of-bounds input index must not cause an error"
        );
        assert!(
            result.unwrap().is_none(),
            "Out-of-bounds input index must not produce a candidate"
        );

        let quote_bad_output_index = make_quote(0, 99, Some(make_quote_value(F1, F1, F1)), true);
        let result = try_build_candidate(&order, &quote_bad_output_index, token_a, token_b);
        assert!(
            result.is_ok(),
            "Out-of-bounds output index must not cause an error"
        );
        assert!(
            result.unwrap().is_none(),
            "Out-of-bounds output index must not produce a candidate"
        );
    }

    #[tokio::test]
    async fn test_build_take_order_candidates_for_correct_direction() {
        let setup = setup_test().await;

        let vault_id = B256::from(U256::from(1u64));
        fund_standard_two_token_vault(&setup, vault_id).await;

        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup), inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            std::slice::from_ref(&raindex_order),
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

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
        fund_standard_two_token_vault(&setup, vault_id).await;

        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup), inputs, outputs).await;

        let fake_token = Address::from([0xABu8; 20]);
        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order],
            fake_token,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should not fail, just return empty");

        assert_eq!(
            candidates.len(),
            0,
            "Expected 0 candidates for non-existent token direction"
        );
    }

    #[tokio::test]
    async fn test_build_take_order_candidates_filters_zero_capacity() {
        let setup = setup_test().await;

        let vault_id = B256::from(U256::from(1u64));
        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup), inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order],
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should not fail, just filter out zero capacity");

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
        fund_standard_two_token_vault(&setup, vault_id).await;

        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let dotrain = create_dotrain_config(&setup);
        let raindex_order1 =
            make_raindex_order(&setup, dotrain.clone(), inputs.clone(), outputs.clone()).await;
        let raindex_order2 = make_raindex_order(&setup, dotrain, inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order1, raindex_order2],
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

        assert_eq!(
            candidates.len(),
            2,
            "Expected 2 candidates from 2 orders with capacity"
        );
    }

    #[tokio::test]
    async fn test_mixed_orders_some_with_capacity_some_without() {
        let setup = setup_test().await;

        let vault_id_with_balance = B256::from(U256::from(1u64));
        let vault_id_empty = B256::from(U256::from(2u64));

        fund_standard_two_token_vault(&setup, vault_id_with_balance).await;

        let vault1_with_balance = create_vault(vault_id_with_balance, &setup, &setup.token1_sg);
        let vault2_with_balance = create_vault(vault_id_with_balance, &setup, &setup.token2_sg);

        let mut vault1_empty = create_vault(vault_id_empty, &setup, &setup.token1_sg);
        vault1_empty.balance = SgBytes(Float::zero().unwrap().as_hex());
        let mut vault2_empty = create_vault(vault_id_empty, &setup, &setup.token2_sg);
        vault2_empty.balance = SgBytes(Float::zero().unwrap().as_hex());

        let raindex_order1 = make_raindex_order(
            &setup,
            create_dotrain_config_with_vault_id(&setup, "0x01"),
            vec![vault1_with_balance.clone(), vault2_with_balance.clone()],
            vec![vault1_with_balance.clone(), vault2_with_balance.clone()],
        )
        .await;

        let raindex_order2 = make_raindex_order(
            &setup,
            create_dotrain_config_with_vault_id(&setup, "0x02"),
            vec![vault1_empty.clone(), vault2_empty.clone()],
            vec![vault1_empty.clone(), vault2_empty.clone()],
        )
        .await;

        let order1_hash = raindex_order1.order_hash();

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order1, raindex_order2],
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

        assert_eq!(
            candidates.len(),
            1,
            "Expected exactly 1 candidate (only the one with capacity)"
        );

        let candidate = &candidates[0];
        let candidate_order_hash = alloy::primitives::keccak256(candidate.order.abi_encode());
        assert_eq!(
            candidate_order_hash, order1_hash,
            "Candidate should be from the order with capacity"
        );
        assert!(
            candidate.max_output.gt(Float::zero().unwrap()).unwrap(),
            "Candidate max_output should be > 0"
        );
    }

    #[tokio::test]
    async fn test_max_output_and_ratio_match_expected_quote_values() {
        let setup = setup_test().await;

        let vault_id = B256::from(U256::from(1u64));
        fund_standard_two_token_vault(&setup, vault_id).await;

        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup), inputs, outputs).await;

        let quotes = raindex_order
            .get_quotes(None, None)
            .await
            .expect("Should get quotes");

        let relevant_quote = quotes
            .iter()
            .find(|q| {
                q.success
                    && q.data.is_some()
                    && q.data
                        .as_ref()
                        .unwrap()
                        .max_output
                        .gt(Float::zero().unwrap())
                        .unwrap()
            })
            .expect("Should have at least one successful quote with capacity");

        let expected_max_output = relevant_quote.data.as_ref().unwrap().max_output;
        let expected_ratio = relevant_quote.data.as_ref().unwrap().ratio;

        let candidates = build_take_order_candidates_for_pair(
            std::slice::from_ref(&raindex_order),
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

        assert_eq!(candidates.len(), 1, "Expected exactly 1 candidate");

        let candidate = &candidates[0];

        assert!(
            candidate.max_output.eq(expected_max_output).unwrap(),
            "Candidate max_output ({:?}) should match expected from quote ({:?})",
            candidate.max_output.format(),
            expected_max_output.format()
        );
        assert!(
            candidate.ratio.eq(expected_ratio).unwrap(),
            "Candidate ratio ({:?}) should match expected from quote ({:?})",
            candidate.ratio.format(),
            expected_ratio.format()
        );

        let expected_max_output_100 = Float::parse("100".to_string()).unwrap();
        let expected_ratio_2 = Float::parse("2".to_string()).unwrap();

        assert!(
            candidate.max_output.eq(expected_max_output_100).unwrap(),
            "max_output should be 100 (from rainlang: amount price: 100 2;)"
        );
        assert!(
            candidate.ratio.eq(expected_ratio_2).unwrap(),
            "ratio should be 2 (from rainlang: amount price: 100 2;)"
        );
    }

    #[tokio::test]
    async fn test_multiple_orders_only_matching_pair_included() {
        let mut setup = setup_test().await;

        let token3 = setup
            .local_evm
            .deploy_new_token("Token3", "Token3", 18, U256::MAX, setup.owner)
            .await;
        let token3_sg = SgErc20 {
            id: SgBytes(token3.address().to_string()),
            address: SgBytes(token3.address().to_string()),
            name: Some("Token3".to_string()),
            symbol: Some("Token3".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };

        let vault_id = B256::from(U256::from(1u64));
        let amount = standard_deposit_amount();
        fund_standard_two_token_vault(&setup, vault_id).await;
        setup
            .local_evm
            .deposit(setup.owner, *token3.address(), amount, 18, vault_id)
            .await;

        let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
        let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);
        let vault3 = create_vault(vault_id, &setup, &token3_sg);

        let raindex_order_a = make_raindex_order(
            &setup,
            create_dotrain_config(&setup),
            vec![vault1.clone(), vault2.clone()],
            vec![vault1.clone(), vault2.clone()],
        )
        .await;

        let dotrain_t1_t3 = format!(
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
    t3:
        network: test-network
        address: {token3}
        decimals: 18
        label: Token3
        symbol: Token3
orderbook:
    test-orderbook:
        address: {orderbook}
orders:
    test-order:
        inputs:
            - token: t1
        outputs:
            - token: t3
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
            token3 = token3.address(),
            spec_version = SpecVersion::current(),
        );

        let raindex_order_b = make_raindex_order(
            &setup,
            dotrain_t1_t3,
            vec![vault1.clone()],
            vec![vault3.clone()],
        )
        .await;

        let order_a_hash = raindex_order_a.order_hash();

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order_a, raindex_order_b],
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

        assert_eq!(
            candidates.len(),
            1,
            "Expected exactly 1 candidate for token1->token2 direction"
        );

        let candidate = &candidates[0];
        let candidate_order_hash = alloy::primitives::keccak256(candidate.order.abi_encode());
        assert_eq!(
            candidate_order_hash, order_a_hash,
            "Candidate should be from order A (token1->token2)"
        );
    }

    #[tokio::test]
    async fn test_single_order_multiple_io_pairs_same_tokens() {
        let setup = setup_test().await;

        let vault_id_1 = B256::from(U256::from(1u64));
        let vault_id_2 = B256::from(U256::from(2u64));

        fund_standard_two_token_vault(&setup, vault_id_1).await;
        fund_standard_two_token_vault(&setup, vault_id_2).await;

        let dotrain_multi_io = format!(
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
              vault-id: 0x01
            - token: t1
              vault-id: 0x02
        outputs:
            - token: t2
              vault-id: 0x01
            - token: t2
              vault-id: 0x02
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
        );

        let vault1_t1 = create_vault(vault_id_1, &setup, &setup.token1_sg);
        let vault1_t2 = create_vault(vault_id_1, &setup, &setup.token2_sg);
        let vault2_t1 = create_vault(vault_id_2, &setup, &setup.token1_sg);
        let vault2_t2 = create_vault(vault_id_2, &setup, &setup.token2_sg);

        let inputs = vec![vault1_t1.clone(), vault2_t1.clone()];
        let outputs = vec![vault1_t2.clone(), vault2_t2.clone()];

        let raindex_order = make_raindex_order(&setup, dotrain_multi_io, inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            std::slice::from_ref(&raindex_order),
            setup.token1,
            setup.token2,
            None,
            None,
        )
        .await
        .expect("Should build candidates");

        assert!(
            candidates.len() >= 2,
            "Expected at least 2 candidates from multiple IO pairs with same tokens, got {}",
            candidates.len()
        );

        let mut seen_io_pairs: std::collections::HashSet<(u32, u32)> =
            std::collections::HashSet::new();
        for candidate in &candidates {
            let io_pair = (candidate.input_io_index, candidate.output_io_index);
            assert!(
                seen_io_pairs.insert(io_pair),
                "Duplicate IO pair found: ({}, {})",
                io_pair.0,
                io_pair.1
            );

            let order_input_token =
                candidate.order.validInputs[candidate.input_io_index as usize].token;
            let order_output_token =
                candidate.order.validOutputs[candidate.output_io_index as usize].token;
            assert_eq!(
                order_input_token, setup.token1,
                "Candidate input token should match requested input"
            );
            assert_eq!(
                order_output_token, setup.token2,
                "Candidate output token should match requested output"
            );
        }
    }

    #[test]
    fn test_try_build_candidate_with_quote_data_none() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, None, true);

        let result = try_build_candidate(&order, &quote, token_a, token_b).unwrap();

        assert!(
            result.is_none(),
            "Should return None when quote.data is None even if success is true"
        );
    }
}
