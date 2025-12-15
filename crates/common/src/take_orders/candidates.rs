use crate::raindex_client::order_quotes::RaindexOrderQuote;
use crate::raindex_client::orders::RaindexOrder;
use crate::raindex_client::RaindexError;
use alloy::primitives::Address;
use futures::StreamExt;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;
#[cfg(target_family = "wasm")]
use std::str::FromStr;

const DEFAULT_QUOTE_CONCURRENCY: usize = 5;

fn indices_in_bounds(order: &OrderV4, input_index: u32, output_index: u32) -> bool {
    (input_index as usize) < order.validInputs.len()
        && (output_index as usize) < order.validOutputs.len()
}

fn matches_direction(
    order: &OrderV4,
    input_index: u32,
    output_index: u32,
    input_token: Address,
    output_token: Address,
) -> bool {
    let order_input_token = order.validInputs[input_index as usize].token;
    let order_output_token = order.validOutputs[output_index as usize].token;
    order_input_token == input_token && order_output_token == output_token
}

fn has_capacity(
    data: &crate::raindex_client::order_quotes::RaindexOrderQuoteValue,
) -> Result<bool, RaindexError> {
    Ok(data.max_output.gt(Float::zero()?)?)
}

#[derive(Clone, Debug)]
pub struct TakeOrderCandidate {
    pub orderbook: Address,
    pub order: OrderV4,
    pub input_io_index: u32,
    pub output_io_index: u32,
    pub max_output: Float,
    pub ratio: Float,
}

fn get_orderbook_address(order: &RaindexOrder) -> Result<Address, RaindexError> {
    #[cfg(target_family = "wasm")]
    {
        Ok(Address::from_str(&order.orderbook())?)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        Ok(order.orderbook())
    }
}

fn build_candidates_for_order(
    order: &RaindexOrder,
    quotes: Vec<RaindexOrderQuote>,
    input_token: Address,
    output_token: Address,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    let order_v4: OrderV4 = order.try_into()?;
    let orderbook = get_orderbook_address(order)?;

    quotes
        .iter()
        .map(|quote| try_build_candidate(orderbook, &order_v4, quote, input_token, output_token))
        .collect::<Result<Vec<_>, _>>()
        .map(|opts| opts.into_iter().flatten().collect())
}

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

    orders
        .iter()
        .zip(quote_results)
        .map(|(order, quotes_result)| {
            build_candidates_for_order(order, quotes_result?, input_token, output_token)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|vecs| vecs.into_iter().flatten().collect())
}

fn try_build_candidate(
    orderbook: Address,
    order: &OrderV4,
    quote: &RaindexOrderQuote,
    input_token: Address,
    output_token: Address,
) -> Result<Option<TakeOrderCandidate>, RaindexError> {
    let data = match (quote.success, &quote.data) {
        (true, Some(d)) => d,
        _ => return Ok(None),
    };

    let input_io_index = quote.pair.input_index;
    let output_io_index = quote.pair.output_index;

    if !indices_in_bounds(order, input_io_index, output_io_index) {
        return Ok(None);
    }

    if !matches_direction(
        order,
        input_io_index,
        output_io_index,
        input_token,
        output_token,
    ) {
        return Ok(None);
    }

    if !has_capacity(data)? {
        return Ok(None);
    }

    Ok(Some(TakeOrderCandidate {
        orderbook,
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
    use crate::raindex_client::orders::RaindexOrder;
    use crate::raindex_client::RaindexClient;
    use crate::test_helpers::dotrain::{
        create_dotrain_config, create_dotrain_config_with_vault_id,
    };
    use crate::test_helpers::local_evm::{
        create_vault, fund_standard_two_token_vault, setup_test as base_setup_test,
        standard_deposit_amount, TestSetup as BaseTestSetup,
    };
    use crate::test_helpers::orders::{deploy::deploy_order, make_basic_order};
    use crate::test_helpers::quotes::{make_quote, make_quote_value};
    use crate::test_helpers::subgraph::create_sg_order;
    use alloy::primitives::{Address, B256, U256};
    use alloy::sol_types::SolValue;
    use rain_math_float::Float;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_subgraph_client::types::common::{SgBigInt, SgBytes, SgErc20, SgVault};
    use std::rc::Rc;

    #[test]
    fn test_indices_in_bounds_valid() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(indices_in_bounds(&order, 0, 0));
    }

    #[test]
    fn test_indices_in_bounds_input_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 99, 0));
    }

    #[test]
    fn test_indices_in_bounds_output_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 0, 99));
    }

    #[test]
    fn test_indices_in_bounds_both_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 99, 99));
    }

    #[test]
    fn test_matches_direction_correct() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(matches_direction(&order, 0, 0, token_a, token_b));
    }

    #[test]
    fn test_matches_direction_wrong_input() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let token_c = Address::from([6u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_c, token_b));
    }

    #[test]
    fn test_matches_direction_wrong_output() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let token_c = Address::from([6u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_a, token_c));
    }

    #[test]
    fn test_matches_direction_reversed() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_b, token_a));
    }

    fn make_quote_value_for_capacity_test(
        max_output: Float,
    ) -> crate::raindex_client::order_quotes::RaindexOrderQuoteValue {
        let zero = Float::zero().unwrap();
        crate::raindex_client::order_quotes::RaindexOrderQuoteValue {
            max_output,
            formatted_max_output: "0".to_string(),
            max_input: zero,
            formatted_max_input: "0".to_string(),
            ratio: zero,
            formatted_ratio: "0".to_string(),
            inverse_ratio: zero,
            formatted_inverse_ratio: "0".to_string(),
        }
    }

    #[test]
    fn test_has_capacity_positive() {
        let positive = Float::parse("100".to_string()).unwrap();
        let data = make_quote_value_for_capacity_test(positive);
        assert!(has_capacity(&data).unwrap());
    }

    #[test]
    fn test_has_capacity_zero() {
        let zero = Float::zero().unwrap();
        let data = make_quote_value_for_capacity_test(zero);
        assert!(!has_capacity(&data).unwrap());
    }

    #[test]
    fn test_has_capacity_negative() {
        let negative = Float::parse("-1".to_string()).unwrap();
        let data = make_quote_value_for_capacity_test(negative);
        assert!(!has_capacity(&data).unwrap());
    }

    struct TestSetup {
        base: BaseTestSetup,
        raindex_client: Rc<RaindexClient>,
    }

    async fn setup_test() -> TestSetup {
        let base = base_setup_test().await;

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
            rpc_url = base.local_evm.url(),
            orderbook = base.orderbook,
            deployer = base.local_evm.deployer.address(),
            token1 = base.token1,
            token2 = base.token2,
        );

        let raindex_client = Rc::new(RaindexClient::new(vec![yaml], None).expect("Valid yaml"));

        TestSetup {
            base,
            raindex_client,
        }
    }

    async fn make_raindex_order(
        setup: &TestSetup,
        dotrain: String,
        inputs: Vec<SgVault>,
        outputs: Vec<SgVault>,
    ) -> RaindexOrder {
        let (order_bytes, order_hash) = deploy_order(&setup.base, dotrain).await;
        let sg_order = create_sg_order(&setup.base, order_bytes, order_hash, inputs, outputs);
        RaindexOrder::try_from_sg_order(Rc::clone(&setup.raindex_client), 123, sg_order, None)
            .expect("Should create RaindexOrder")
    }

    #[test]
    fn test_try_build_candidate_wrong_direction() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(f1, f1, f1)), true);

        let result = try_build_candidate(orderbook, &order, &quote, token_b, token_a).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_zero_capacity() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let zero = Float::zero().unwrap();
        let f1 = Float::parse("1".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(zero, zero, f1)), true);

        let result = try_build_candidate(orderbook, &order, &quote, token_a, token_b).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_success() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();
        let f2 = Float::parse("2".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(f2, f1, f1)), true);

        let result = try_build_candidate(orderbook, &order, &quote, token_a, token_b).unwrap();

        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.orderbook, orderbook);
        assert_eq!(candidate.input_io_index, 0);
        assert_eq!(candidate.output_io_index, 0);
        assert!(candidate.max_output.eq(f2).unwrap());
    }

    #[test]
    fn test_try_build_candidate_failed_quote() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, None, false);

        let result = try_build_candidate(orderbook, &order, &quote, token_a, token_b);

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
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();

        let quote_bad_input_index = make_quote(99, 0, Some(make_quote_value(f1, f1, f1)), true);
        let result =
            try_build_candidate(orderbook, &order, &quote_bad_input_index, token_a, token_b);
        assert!(
            result.is_ok(),
            "Out-of-bounds input index must not cause an error"
        );
        assert!(
            result.unwrap().is_none(),
            "Out-of-bounds input index must not produce a candidate"
        );

        let quote_bad_output_index = make_quote(0, 99, Some(make_quote_value(f1, f1, f1)), true);
        let result =
            try_build_candidate(orderbook, &order, &quote_bad_output_index, token_a, token_b);
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
        fund_standard_two_token_vault(&setup.base, vault_id).await;

        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup.base), inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            std::slice::from_ref(&raindex_order),
            setup.base.token1,
            setup.base.token2,
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
        fund_standard_two_token_vault(&setup.base, vault_id).await;

        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup.base), inputs, outputs).await;

        let fake_token = Address::from([0xABu8; 20]);
        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order],
            fake_token,
            setup.base.token2,
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
        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup.base), inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order],
            setup.base.token1,
            setup.base.token2,
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
        fund_standard_two_token_vault(&setup.base, vault_id).await;

        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let dotrain = create_dotrain_config(&setup.base);
        let raindex_order1 =
            make_raindex_order(&setup, dotrain.clone(), inputs.clone(), outputs.clone()).await;
        let raindex_order2 = make_raindex_order(&setup, dotrain, inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order1, raindex_order2],
            setup.base.token1,
            setup.base.token2,
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

        fund_standard_two_token_vault(&setup.base, vault_id_with_balance).await;

        let vault1_with_balance =
            create_vault(vault_id_with_balance, &setup.base, &setup.base.token1_sg);
        let vault2_with_balance =
            create_vault(vault_id_with_balance, &setup.base, &setup.base.token2_sg);

        let mut vault1_empty = create_vault(vault_id_empty, &setup.base, &setup.base.token1_sg);
        vault1_empty.balance = SgBytes(Float::zero().unwrap().as_hex());
        let mut vault2_empty = create_vault(vault_id_empty, &setup.base, &setup.base.token2_sg);
        vault2_empty.balance = SgBytes(Float::zero().unwrap().as_hex());

        let raindex_order1 = make_raindex_order(
            &setup,
            create_dotrain_config_with_vault_id(&setup.base, "0x01"),
            vec![vault1_with_balance.clone(), vault2_with_balance.clone()],
            vec![vault1_with_balance.clone(), vault2_with_balance.clone()],
        )
        .await;

        let raindex_order2 = make_raindex_order(
            &setup,
            create_dotrain_config_with_vault_id(&setup.base, "0x02"),
            vec![vault1_empty.clone(), vault2_empty.clone()],
            vec![vault1_empty.clone(), vault2_empty.clone()],
        )
        .await;

        let order1_hash = raindex_order1.order_hash();

        let candidates = build_take_order_candidates_for_pair(
            &[raindex_order1, raindex_order2],
            setup.base.token1,
            setup.base.token2,
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
        fund_standard_two_token_vault(&setup.base, vault_id).await;

        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let inputs = vec![vault1.clone(), vault2.clone()];
        let outputs = vec![vault1.clone(), vault2.clone()];

        let raindex_order =
            make_raindex_order(&setup, create_dotrain_config(&setup.base), inputs, outputs).await;

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
            setup.base.token1,
            setup.base.token2,
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
            .base
            .local_evm
            .deploy_new_token("Token3", "Token3", 18, U256::MAX, setup.base.owner)
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
        fund_standard_two_token_vault(&setup.base, vault_id).await;
        setup
            .base
            .local_evm
            .deposit(setup.base.owner, *token3.address(), amount, 18, vault_id)
            .await;

        let vault1 = create_vault(vault_id, &setup.base, &setup.base.token1_sg);
        let vault2 = create_vault(vault_id, &setup.base, &setup.base.token2_sg);
        let vault3 = create_vault(vault_id, &setup.base, &token3_sg);

        let raindex_order_a = make_raindex_order(
            &setup,
            create_dotrain_config(&setup.base),
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
            rpc_url = setup.base.local_evm.url(),
            orderbook = setup.base.orderbook,
            deployer = setup.base.local_evm.deployer.address(),
            token1 = setup.base.token1,
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
            setup.base.token1,
            setup.base.token2,
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

        fund_standard_two_token_vault(&setup.base, vault_id_1).await;
        fund_standard_two_token_vault(&setup.base, vault_id_2).await;

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
            rpc_url = setup.base.local_evm.url(),
            orderbook = setup.base.orderbook,
            deployer = setup.base.local_evm.deployer.address(),
            token1 = setup.base.token1,
            token2 = setup.base.token2,
            spec_version = SpecVersion::current(),
        );

        let vault1_t1 = create_vault(vault_id_1, &setup.base, &setup.base.token1_sg);
        let vault1_t2 = create_vault(vault_id_1, &setup.base, &setup.base.token2_sg);
        let vault2_t1 = create_vault(vault_id_2, &setup.base, &setup.base.token1_sg);
        let vault2_t2 = create_vault(vault_id_2, &setup.base, &setup.base.token2_sg);

        let inputs = vec![vault1_t1.clone(), vault2_t1.clone()];
        let outputs = vec![vault1_t2.clone(), vault2_t2.clone()];

        let raindex_order = make_raindex_order(&setup, dotrain_multi_io, inputs, outputs).await;

        let candidates = build_take_order_candidates_for_pair(
            std::slice::from_ref(&raindex_order),
            setup.base.token1,
            setup.base.token2,
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
                order_input_token, setup.base.token1,
                "Candidate input token should match requested input"
            );
            assert_eq!(
                order_output_token, setup.base.token2,
                "Candidate output token should match requested output"
            );
        }
    }

    #[test]
    fn test_try_build_candidate_with_quote_data_none() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, None, true);

        let result = try_build_candidate(orderbook, &order, &quote, token_a, token_b).unwrap();

        assert!(
            result.is_none(),
            "Should return None when quote.data is None even if success is true"
        );
    }
}
