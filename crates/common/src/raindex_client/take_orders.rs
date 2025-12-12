use super::orders::{GetOrdersFilters, GetOrdersTokenFilter, RaindexOrder};
use super::*;
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_order_candidates_for_pair, build_take_orders_config_from_sell_simulation, cmp_float,
    simulate_sell_over_candidates, BuiltTakeOrdersConfig, MinReceiveMode, SimulatedSellResult,
    TakeOrderCandidate,
};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use std::collections::HashMap;
use std::ops::Div;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct ParsedTakeOrdersRequest {
    sell_token: Address,
    buy_token: Address,
    sell_amount: Float,
    min_receive_mode: MinReceiveMode,
}

fn parse_request(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    min_receive_mode: MinReceiveMode,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let sell_token = Address::from_str(sell_token)?;
    let buy_token = Address::from_str(buy_token)?;
    let sell_amount = Float::parse(sell_amount.to_string())?;
    Ok(ParsedTakeOrdersRequest {
        sell_token,
        buy_token,
        sell_amount,
        min_receive_mode,
    })
}

async fn build_candidates_for_chain(
    orders: &[RaindexOrder],
    sell_token: Address,
    buy_token: Address,
    block_number: Option<u64>,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    let candidates =
        build_take_order_candidates_for_pair(orders, sell_token, buy_token, block_number, None)
            .await?;
    if candidates.is_empty() {
        return Err(RaindexError::NoLiquidity);
    }
    Ok(candidates)
}

fn worst_price(sim: &SimulatedSellResult) -> Result<Option<Float>, RaindexError> {
    let mut max: Option<Float> = None;
    for leg in &sim.legs {
        let ratio = leg.candidate.ratio;
        match &max {
            None => max = Some(ratio),
            Some(current_max) => {
                if cmp_float(&ratio, current_max)? == std::cmp::Ordering::Greater {
                    max = Some(ratio);
                }
            }
        }
    }
    Ok(max)
}

fn select_best_orderbook_simulation(
    candidates: Vec<TakeOrderCandidate>,
    sell_budget: Float,
) -> Result<(Address, SimulatedSellResult), RaindexError> {
    let mut orderbook_candidates: HashMap<Address, Vec<TakeOrderCandidate>> = HashMap::new();
    for candidate in candidates {
        orderbook_candidates
            .entry(candidate.orderbook)
            .or_default()
            .push(candidate);
    }

    let mut best_result: Option<(Address, SimulatedSellResult)> = None;

    for (orderbook, candidates) in orderbook_candidates {
        let sim = simulate_sell_over_candidates(candidates, sell_budget)?;

        if sim.legs.is_empty() {
            continue;
        }

        let is_better = match &best_result {
            None => true,
            Some((best_addr, best_sim)) => {
                if sim.total_buy_amount.gt(best_sim.total_buy_amount)? {
                    true
                } else if sim.total_buy_amount.eq(best_sim.total_buy_amount)? {
                    let sim_worst = worst_price(&sim)?;
                    let best_worst = worst_price(best_sim)?;
                    match (sim_worst, best_worst) {
                        (Some(sw), Some(bw)) => match cmp_float(&sw, &bw)? {
                            std::cmp::Ordering::Less => true,
                            std::cmp::Ordering::Equal => orderbook < *best_addr,
                            std::cmp::Ordering::Greater => false,
                        },
                        _ => orderbook < *best_addr,
                    }
                } else {
                    false
                }
            }
        };

        if is_better {
            best_result = Some((orderbook, sim));
        }
    }

    best_result.ok_or(RaindexError::NoLiquidity)
}

fn build_calldata_result(
    orderbook: Address,
    built_config: BuiltTakeOrdersConfig,
) -> Result<TakeOrdersCalldataResult, RaindexError> {
    let calldata_bytes = takeOrders3Call {
        config: built_config.config,
    }
    .abi_encode();
    let calldata = Bytes::copy_from_slice(&calldata_bytes);

    let zero = Float::zero()?;
    let effective_price = if built_config.sim.total_buy_amount.gt(zero)? {
        built_config
            .sim
            .total_sell_amount
            .div(built_config.sim.total_buy_amount)?
    } else {
        zero
    };

    let prices: Vec<Float> = built_config
        .sim
        .legs
        .iter()
        .map(|leg| leg.candidate.ratio)
        .collect();

    Ok(TakeOrdersCalldataResult {
        orderbook,
        calldata,
        effective_price,
        prices,
    })
}

impl RaindexClient {
    async fn fetch_orders_for_pair(
        &self,
        chain_id: u32,
        sell_token: Address,
        buy_token: Address,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let filters = GetOrdersFilters {
            owners: vec![],
            active: Some(true),
            order_hash: None,
            tokens: Some(GetOrdersTokenFilter {
                inputs: Some(vec![sell_token]),
                outputs: Some(vec![buy_token]),
            }),
        };

        let orders = self
            .get_orders(Some(ChainIds(vec![chain_id])), Some(filters), None)
            .await?;

        if orders.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        Ok(orders)
    }
}

/// Combined result for generating takeOrders3 calldata and price info.
///
/// `calldata` can be sent directly as transaction data; `effective_price` and
/// `prices` provide blended and per-leg prices (sell per 1 buy).
#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersCalldataResult {
    /// The orderbook contract address to call with this calldata
    #[tsify(type = "Address")]
    pub orderbook: Address,
    /// ABI-encoded calldata for `IOrderBookV5.takeOrders3` (hex in JS)
    #[tsify(type = "Hex")]
    pub calldata: Bytes,
    /// Blended sell per 1 buy (totalSell / totalBuy)
    #[tsify(type = "Hex")]
    pub effective_price: Float,
    /// Per-leg prices (sell per 1 buy), sorted best (cheapest) to worst
    #[tsify(type = "Hex[]")]
    pub prices: Vec<Float>,
}
impl_wasm_traits!(TakeOrdersCalldataResult);

#[wasm_export]
impl RaindexClient {
    /// Generates ABI-encoded calldata for `takeOrders3()` using an exact-in builder.
    ///
    /// Discovers orders for the given `(chainId, sellToken, buyToken)` pair, simulates
    /// how much `buyToken` is received for the exact `sellAmount` budget, applies the
    /// `minReceiveMode` policy, and returns both calldata and price information.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getTakeOrdersCalldata(
    ///   137,                 // chainId
    ///   "0xSELL...",         // sellToken
    ///   "0xBUY...",          // buyToken
    ///   sellAmountFloat,     // exact sell amount as Float hex string
    ///   "partial",           // or "exact"
    /// );
    /// if (result.error) {
    ///   console.error("Failed to build takeOrders calldata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const { calldata, effectivePrice, prices } = result.value;
    /// // Use calldata for transaction, show prices in UI, etc.
    /// ```
    #[wasm_export(
        js_name = "getTakeOrdersCalldata",
        return_description = "Encoded takeOrders3 calldata and price information",
        unchecked_return_type = "TakeOrdersCalldataResult"
    )]
    pub async fn get_take_orders_calldata(
        &self,
        #[wasm_export(
            js_name = "chainId",
            param_description = "Chain ID of the target network"
        )]
        chain_id: u32,
        #[wasm_export(
            js_name = "sellToken",
            param_description = "Token address the taker will GIVE",
            unchecked_param_type = "Address"
        )]
        sell_token: String,
        #[wasm_export(
            js_name = "buyToken",
            param_description = "Token address the taker will RECEIVE",
            unchecked_param_type = "Address"
        )]
        buy_token: String,
        #[wasm_export(
            js_name = "sellAmount",
            param_description = "Exact sell amount as a Float hex string in sellToken units",
            unchecked_param_type = "Hex"
        )]
        sell_amount: String,
        #[wasm_export(
            js_name = "minReceiveMode",
            param_description = "Minimum receive policy: partial or exact"
        )]
        min_receive_mode: MinReceiveMode,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let req = parse_request(&sell_token, &buy_token, &sell_amount, min_receive_mode)?;

        let orders = self
            .fetch_orders_for_pair(chain_id, req.sell_token, req.buy_token)
            .await?;

        let rpc_urls = self.get_rpc_urls_for_chain(chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls)?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates =
            build_candidates_for_chain(&orders, req.sell_token, req.buy_token, Some(block_number))
                .await?;

        let (best_orderbook, best_sim) =
            select_best_orderbook_simulation(candidates, req.sell_amount)?;

        let built = build_take_orders_config_from_sell_simulation(best_sim, req.min_receive_mode)?
            .ok_or(RaindexError::NoLiquidity)?;

        build_calldata_result(best_orderbook, built)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::take_orders::MinReceiveMode;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::{from_js_value, to_js_value};

        #[wasm_bindgen_test]
        fn test_min_receive_mode_serialization() {
            let partial = MinReceiveMode::Partial;
            let exact = MinReceiveMode::Exact;

            let partial_js = to_js_value(&partial).unwrap();
            let exact_js = to_js_value(&exact).unwrap();

            let partial_back: MinReceiveMode = from_js_value(partial_js).unwrap();
            let exact_back: MinReceiveMode = from_js_value(exact_js).unwrap();

            assert!(matches!(partial_back, MinReceiveMode::Partial));
            assert!(matches!(exact_back, MinReceiveMode::Exact));
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::super::{
            build_calldata_result, parse_request, select_best_orderbook_simulation,
        };
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::RaindexClient;
        use crate::raindex_client::RaindexError;
        use crate::take_orders::{build_take_orders_config_from_sell_simulation, MinReceiveMode};
        use crate::test_helpers::candidates::make_candidate;
        use crate::test_helpers::dotrain::create_dotrain_config_for_orderbook;
        use crate::test_helpers::local_evm::{
            create_vault, create_vault_for_orderbook, deposit_to_orderbook,
            setup_multi_orderbook_test, setup_test as base_setup_test, standard_deposit_amount,
        };
        use crate::test_helpers::orders::deploy::deploy_order_to_orderbook;
        use crate::test_helpers::subgraph::{
            create_sg_order_json, create_sg_order_json_with_orderbook, get_minimal_yaml_for_chain,
            get_multi_orderbook_yaml,
        };
        use alloy::network::TransactionBuilder;
        use alloy::primitives::{address, keccak256, Address, B256, U256};
        use alloy::rpc::types::TransactionRequest;
        use alloy::serde::WithOtherFields;
        use alloy::sol_types::{SolCall, SolValue};
        use httpmock::MockServer;
        use rain_math_float::Float;
        use rain_orderbook_bindings::IOrderBookV5::{takeOrders3Call, TakeOrdersConfigV4};
        use serde_json::json;
        use std::ops::Sub;

        #[test]
        fn test_parse_request_valid_inputs() {
            let result = parse_request(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "100",
                MinReceiveMode::Partial,
            );

            assert!(result.is_ok());
            let req = result.unwrap();
            assert_eq!(
                req.sell_token,
                address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            );
            assert_eq!(
                req.buy_token,
                address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            );
            assert!(req
                .sell_amount
                .eq(Float::parse("100".to_string()).unwrap())
                .unwrap());
            assert!(matches!(req.min_receive_mode, MinReceiveMode::Partial));
        }

        #[test]
        fn test_parse_request_invalid_sell_token() {
            let result = parse_request(
                "not-an-address",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "100",
                MinReceiveMode::Partial,
            );

            assert!(matches!(result, Err(RaindexError::FromHexError(_))));
        }

        #[test]
        fn test_parse_request_invalid_buy_token() {
            let result = parse_request(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "invalid",
                "100",
                MinReceiveMode::Partial,
            );

            assert!(matches!(result, Err(RaindexError::FromHexError(_))));
        }

        #[test]
        fn test_parse_request_invalid_sell_amount() {
            let result = parse_request(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "not-a-number",
                MinReceiveMode::Partial,
            );

            assert!(matches!(result, Err(RaindexError::Float(_))));
        }

        #[test]
        fn test_parse_request_exact_mode() {
            let result = parse_request(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "50",
                MinReceiveMode::Exact,
            );

            assert!(result.is_ok());
            let req = result.unwrap();
            assert!(matches!(req.min_receive_mode, MinReceiveMode::Exact));
        }

        #[test]
        fn test_build_calldata_result_produces_valid_calldata() {
            let ob = Address::from([0x11u8; 20]);
            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("2".to_string()).unwrap();
            let candidate = make_candidate(ob, max_output, ratio);
            let candidates = vec![candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let (_, sim) = select_best_orderbook_simulation(candidates, sell_budget).unwrap();
            let built = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
                .unwrap()
                .unwrap();

            let result = build_calldata_result(ob, built);

            assert!(result.is_ok());
            let calldata_result = result.unwrap();
            assert_eq!(calldata_result.orderbook, ob);
            assert!(!calldata_result.calldata.is_empty());
            assert!(!calldata_result.prices.is_empty());

            let decoded = takeOrders3Call::abi_decode(&calldata_result.calldata)
                .expect("Should decode calldata");
            assert!(!decoded.config.orders.is_empty());
        }

        #[test]
        fn test_build_calldata_result_effective_price_calculation() {
            let ob = Address::from([0x11u8; 20]);
            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("2".to_string()).unwrap();
            let candidate = make_candidate(ob, max_output, ratio);
            let candidates = vec![candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let (_, sim) = select_best_orderbook_simulation(candidates, sell_budget).unwrap();
            let built = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
                .unwrap()
                .unwrap();

            let result = build_calldata_result(ob, built).unwrap();

            let zero = Float::zero().unwrap();
            assert!(
                result.effective_price.gt(zero).unwrap(),
                "Effective price should be > 0"
            );
        }

        #[test]
        fn test_build_calldata_result_prices_match_legs() {
            let ob = Address::from([0x11u8; 20]);
            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("2".to_string()).unwrap();
            let candidate = make_candidate(ob, max_output, ratio);
            let candidates = vec![candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let (_, sim) = select_best_orderbook_simulation(candidates, sell_budget).unwrap();
            let leg_count = sim.legs.len();
            let built = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
                .unwrap()
                .unwrap();

            let result = build_calldata_result(ob, built).unwrap();

            assert_eq!(
                result.prices.len(),
                leg_count,
                "Number of prices should match number of legs"
            );
            assert!(
                result.prices[0].eq(ratio).unwrap(),
                "Price should match the candidate ratio"
            );
        }

        #[test]
        fn test_select_best_orderbook_single_orderbook() {
            let ob1 = Address::from([0x11u8; 20]);
            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("2".to_string()).unwrap();
            let candidate = make_candidate(ob1, max_output, ratio);
            let candidates = vec![candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (addr, sim) = result.unwrap();
            assert_eq!(addr, ob1);
            assert!(!sim.legs.is_empty());
            assert!(sim.total_buy_amount.gt(Float::zero().unwrap()).unwrap());
            assert!(sim.total_sell_amount.gt(Float::zero().unwrap()).unwrap());
        }

        #[test]
        fn test_select_best_orderbook_multiple_books_picks_best() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("5".to_string()).unwrap();
            let ob1_ratio = Float::parse("1".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("8".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();
            assert_eq!(winner, ob2);
            let expected_buy = Float::parse("8".to_string()).unwrap();
            assert!(sim.total_buy_amount.eq(expected_buy).unwrap());
        }

        #[test]
        fn test_select_best_orderbook_skips_empty_sims() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("10".to_string()).unwrap();
            let ob1_ratio = Float::parse("2".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("5".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::zero().unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(matches!(result, Err(RaindexError::NoLiquidity)));
        }

        #[test]
        fn test_select_best_orderbook_all_empty_returns_no_liquidity() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("10".to_string()).unwrap();
            let ob1_ratio = Float::parse("2".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("5".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::zero().unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_err());
            assert!(matches!(result, Err(RaindexError::NoLiquidity)));
        }

        #[test]
        fn test_select_best_orderbook_skips_empty_picks_valid() {
            let ob_empty = Address::from([0x11u8; 20]);
            let ob_valid = Address::from([0x22u8; 20]);

            let empty_max_output = Float::parse("10".to_string()).unwrap();
            let empty_ratio = Float::parse("1000000".to_string()).unwrap();
            let empty_candidate = make_candidate(ob_empty, empty_max_output, empty_ratio);

            let valid_max_output = Float::parse("5".to_string()).unwrap();
            let valid_ratio = Float::parse("1".to_string()).unwrap();
            let valid_candidate = make_candidate(ob_valid, valid_max_output, valid_ratio);

            let candidates = vec![empty_candidate, valid_candidate];
            let sell_budget = Float::parse("10".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();
            assert_eq!(winner, ob_valid);
            assert!(!sim.legs.is_empty());
            assert!(sim.total_buy_amount.gt(Float::zero().unwrap()).unwrap());
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_no_orders_returns_no_liquidity() {
            let sg_server = MockServer::start_async().await;

            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": []
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": []
                    }
                }));
            });

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    "http://localhost:0/unused_rpc1",
                    "http://localhost:0/unused_rpc2",
                )],
                None,
            )
            .unwrap();

            let res = client
                .get_take_orders_calldata(
                    1,
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                    "1".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when subgraph returns empty orders, got: {:?}",
                res
            );
        }

        use crate::test_helpers::dotrain::create_dotrain_config_with_params;
        use crate::test_helpers::local_evm::fund_standard_two_token_vault;
        use crate::test_helpers::orders::deploy::deploy_order;

        #[tokio::test]
        async fn test_get_take_orders_calldata_no_candidates_returns_no_liquidity() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let res = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when no candidates (no vault balance), got: {:?}",
                res
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_happy_path_returns_valid_config() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should succeed with funded vault and valid order");

            assert_eq!(
                result.orderbook, setup.orderbook,
                "Orderbook address should match"
            );

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert!(
                !config.orders.is_empty(),
                "Should have at least one order in config"
            );

            assert_eq!(
                config.minimumInput,
                Float::zero().unwrap().get_inner(),
                "minimumInput should be zero for Partial mode"
            );

            assert!(
                !result.prices.is_empty(),
                "Should have at least one price in result"
            );

            let expected_ratio = Float::parse("2".to_string()).unwrap();
            assert!(
                result.prices[0].eq(expected_ratio).unwrap(),
                "Price should match expected ratio of 2, got: {:?}",
                result.prices[0].format()
            );

            let zero = Float::zero().unwrap();
            assert!(
                result.effective_price.gt(zero).unwrap(),
                "Effective price should be > 0"
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_min_receive_mode_exact_vs_partial() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result_partial = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Partial mode should succeed");

            let result_exact = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Exact,
                )
                .await
                .expect("Exact mode should succeed");

            let decoded_partial = takeOrders3Call::abi_decode(&result_partial.calldata)
                .expect("Should decode partial calldata");
            let config_partial = decoded_partial.config;

            let decoded_exact = takeOrders3Call::abi_decode(&result_exact.calldata)
                .expect("Should decode exact calldata");
            let config_exact = decoded_exact.config;

            assert_eq!(
                config_partial.maximumInput, config_exact.maximumInput,
                "maximumInput should be the same for both modes"
            );

            assert_eq!(
                config_partial.minimumInput,
                Float::zero().unwrap().get_inner(),
                "minimumInput should be zero for Partial mode"
            );

            assert_eq!(
                config_exact.minimumInput, config_exact.maximumInput,
                "minimumInput should equal maximumInput for Exact mode"
            );

            assert_eq!(
                config_partial.maximumIORatio, config_exact.maximumIORatio,
                "maximumIORatio should be the same for both modes"
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_wrong_direction_returns_no_liquidity() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let fake_token = "0xcccccccccccccccccccccccccccccccccccccccc";
            let res = client
                .get_take_orders_calldata(
                    123,
                    fake_token.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when using wrong direction/fake token, got: {:?}",
                res
            );
        }

        #[tokio::test]
        async fn test_min_receive_mode_exact_reverts_when_simulated_buy_cannot_be_met() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config_with_params(&setup, "50", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result_partial = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Partial mode calldata build should succeed");

            let result_exact = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Exact,
                )
                .await
                .expect("Exact mode calldata build should succeed");

            let decoded_partial = takeOrders3Call::abi_decode(&result_partial.calldata)
                .expect("Should decode partial calldata");
            let config_partial = decoded_partial.config;

            let decoded_exact = takeOrders3Call::abi_decode(&result_exact.calldata)
                .expect("Should decode exact calldata");
            let config_exact = decoded_exact.config;

            assert_eq!(
                config_partial.minimumInput,
                Float::zero().unwrap().get_inner(),
                "Partial mode minimumInput should be zero"
            );
            assert_eq!(
                config_exact.minimumInput, config_exact.maximumInput,
                "Exact mode minimumInput should equal maximumInput"
            );
            assert_eq!(
                config_partial.maximumInput, config_exact.maximumInput,
                "Both modes should have the same maximumInput (simulated total buy)"
            );

            let withdraw_amount =
                Float::from_fixed_decimal(standard_deposit_amount() - U256::from(1), 18)
                    .unwrap()
                    .get_inner();

            let withdraw_tx = setup
                .local_evm
                .orderbook
                .withdraw3(setup.token2, vault_id, withdraw_amount, vec![])
                .from(setup.owner)
                .into_transaction_request();
            setup
                .local_evm
                .send_transaction(withdraw_tx)
                .await
                .expect("Withdraw should succeed");

            let taker = setup.local_evm.signer_wallets[1].default_signer().address();
            let taker_token1_balance = U256::from(10).pow(U256::from(22));
            let token1_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token1)
                .unwrap();

            token1_contract
                .transfer(taker, taker_token1_balance)
                .from(setup.owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            token1_contract
                .approve(setup.orderbook, taker_token1_balance)
                .from(taker)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            let exact_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(result_exact.calldata.to_vec())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let exact_call_result = setup.local_evm.call(exact_tx.clone()).await;
            assert!(
                exact_call_result.is_err(),
                "Exact mode should revert when simulated buy cannot be met, but got: {:?}",
                exact_call_result
            );

            let error_str = format!("{:?}", exact_call_result.unwrap_err());
            assert!(
                error_str.contains("MinimumInput") || error_str.contains("execution reverted"),
                "Error should indicate MinimumInput revert, got: {}",
                error_str
            );

            let partial_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(result_partial.calldata.to_vec())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let partial_call_result = setup.local_evm.call(partial_tx.clone()).await;
            assert!(
                partial_call_result.is_ok(),
                "Partial mode should NOT revert (may succeed with smaller totals or be a no-op), but got: {:?}",
                partial_call_result
            );

            let partial_tx_result = setup.local_evm.send_transaction(partial_tx).await;
            assert!(
                partial_tx_result.is_ok(),
                "Partial mode transaction should succeed, but got: {:?}",
                partial_tx_result
            );
        }

        use crate::test_helpers::dotrain::create_dotrain_config_with_vault_and_ratio;

        #[tokio::test]
        async fn test_maximum_io_ratio_enforcement_skips_overpriced_leg() {
            let setup = base_setup_test().await;

            let vault_id_1 = B256::from(U256::from(1u64));
            let vault_id_2 = B256::from(U256::from(2u64));

            let amount = standard_deposit_amount();
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_1)
                .await;
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_2)
                .await;

            let dotrain_cheap =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x01", "50", "1");
            let dotrain_expensive =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x02", "50", "2");

            let (order_bytes_cheap, order_hash_cheap) = deploy_order(&setup, dotrain_cheap).await;
            let (order_bytes_expensive, order_hash_expensive) =
                deploy_order(&setup, dotrain_expensive).await;

            let vault1 = create_vault(vault_id_1, &setup, &setup.token2_sg);
            let vault2 = create_vault(vault_id_2, &setup, &setup.token2_sg);
            let input_vault = create_vault(vault_id_1, &setup, &setup.token1_sg);

            let sg_order_cheap = create_sg_order_json(
                &setup,
                &order_bytes_cheap,
                order_hash_cheap,
                vec![input_vault.clone()],
                vec![vault1.clone()],
            );
            let sg_order_expensive = create_sg_order_json(
                &setup,
                &order_bytes_expensive,
                order_hash_expensive,
                vec![input_vault.clone()],
                vec![vault2.clone()],
            );

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_cheap, sg_order_expensive]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "200".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should build calldata with both orders");

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let original_config = decoded.config;

            assert_eq!(
                original_config.orders.len(),
                2,
                "Should have 2 orders in config"
            );

            let worst_ratio = Float::parse("2".to_string()).unwrap();
            assert_eq!(
                original_config.maximumIORatio,
                worst_ratio.get_inner(),
                "maximumIORatio should equal worst simulated price (2)"
            );

            assert_eq!(result.prices.len(), 2, "Should have 2 prices");
            let cheap_price = Float::parse("1".to_string()).unwrap();
            let expensive_price = Float::parse("2".to_string()).unwrap();
            assert!(
                result.prices.iter().any(|p| p.eq(cheap_price).unwrap()),
                "Should have price 1 in the list"
            );
            assert!(
                result.prices.iter().any(|p| p.eq(expensive_price).unwrap()),
                "Should have price 2 in the list"
            );

            let lowered_max_io_ratio = Float::parse("1.5".to_string()).unwrap();
            let modified_config = TakeOrdersConfigV4 {
                minimumInput: original_config.minimumInput,
                maximumInput: original_config.maximumInput,
                maximumIORatio: lowered_max_io_ratio.get_inner(),
                orders: original_config.orders.clone(),
                data: original_config.data.clone(),
            };

            let modified_calldata_bytes = takeOrders3Call {
                config: modified_config,
            }
            .abi_encode();

            let taker = setup.local_evm.signer_wallets[1].default_signer().address();
            let taker_balance = U256::from(10).pow(U256::from(22));
            let token1_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token1)
                .unwrap();

            token1_contract
                .transfer(taker, taker_balance)
                .from(setup.owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            token1_contract
                .approve(setup.orderbook, taker_balance)
                .from(taker)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            let tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(modified_calldata_bytes.clone())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let call_result = setup.local_evm.call(tx.clone()).await;
            assert!(
                call_result.is_ok(),
                "Partial mode with lowered maximumIORatio should not revert, got: {:?}",
                call_result
            );

            let token2_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token2)
                .unwrap();

            let taker_token2_before: U256 = token2_contract.balanceOf(taker).call().await.unwrap();

            let tx_result = setup.local_evm.send_transaction(tx).await;
            assert!(
                tx_result.is_ok(),
                "Transaction should succeed, got: {:?}",
                tx_result
            );

            let taker_token2_after: U256 = token2_contract.balanceOf(taker).call().await.unwrap();

            let received = taker_token2_after - taker_token2_before;
            let expected_from_cheap_only =
                Float::from_fixed_decimal(U256::from(50) * U256::from(10).pow(U256::from(18)), 18)
                    .unwrap();

            let received_float = Float::from_fixed_decimal(received, 18).unwrap();
            assert!(
                received_float.lte(expected_from_cheap_only).unwrap(),
                "Should only receive from cheap order (max 50), got: {:?}",
                received_float.format()
            );

            assert!(
                received > U256::ZERO,
                "Should have received some tokens from cheap order"
            );

            let exact_config = TakeOrdersConfigV4 {
                minimumInput: original_config.maximumInput,
                maximumInput: original_config.maximumInput,
                maximumIORatio: lowered_max_io_ratio.get_inner(),
                orders: original_config.orders.clone(),
                data: original_config.data.clone(),
            };

            let exact_calldata_bytes = takeOrders3Call {
                config: exact_config,
            }
            .abi_encode();

            let exact_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(exact_calldata_bytes)
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let exact_call_result = setup.local_evm.call(exact_tx).await;
            assert!(
                exact_call_result.is_err(),
                "Exact mode should revert when expensive leg is skipped due to maximumIORatio, got: {:?}",
                exact_call_result
            );

            let error_str = format!("{:?}", exact_call_result.unwrap_err());
            assert!(
                error_str.contains("MinimumInput") || error_str.contains("execution reverted"),
                "Error should indicate MinimumInput revert because expected buy cannot be met, got: {}",
                error_str
            );
        }

        #[tokio::test]
        async fn test_maximum_io_ratio_enforcement_with_worsened_on_chain_price() {
            let setup = base_setup_test().await;

            let vault_id_1 = B256::from(U256::from(1u64));
            let vault_id_2 = B256::from(U256::from(2u64));

            let amount = standard_deposit_amount();
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_1)
                .await;
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_2)
                .await;

            let dotrain_cheap =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x01", "50", "1");
            let dotrain_expensive =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x02", "50", "2");

            let (order_bytes_cheap, order_hash_cheap) = deploy_order(&setup, dotrain_cheap).await;
            let (order_bytes_expensive, order_hash_expensive) =
                deploy_order(&setup, dotrain_expensive.clone()).await;

            let vault1 = create_vault(vault_id_1, &setup, &setup.token2_sg);
            let vault2 = create_vault(vault_id_2, &setup, &setup.token2_sg);
            let input_vault = create_vault(vault_id_1, &setup, &setup.token1_sg);

            let sg_order_cheap = create_sg_order_json(
                &setup,
                &order_bytes_cheap,
                order_hash_cheap,
                vec![input_vault.clone()],
                vec![vault1.clone()],
            );
            let sg_order_expensive = create_sg_order_json(
                &setup,
                &order_bytes_expensive,
                order_hash_expensive,
                vec![input_vault.clone()],
                vec![vault2.clone()],
            );

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_cheap, sg_order_expensive]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "200".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should build calldata with both orders");

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let original_config = decoded.config;

            let worst_ratio = Float::parse("2".to_string()).unwrap();
            assert_eq!(
                original_config.maximumIORatio,
                worst_ratio.get_inner(),
                "maximumIORatio should equal worst simulated price (2)"
            );

            let withdraw_amount = Float::from_fixed_decimal(amount, 18).unwrap().get_inner();

            let withdraw_tx = setup
                .local_evm
                .orderbook
                .withdraw3(setup.token2, vault_id_2, withdraw_amount, vec![])
                .from(setup.owner)
                .into_transaction_request();
            setup
                .local_evm
                .send_transaction(withdraw_tx)
                .await
                .expect("Withdraw should succeed");

            let vault_id_3 = B256::from(U256::from(3u64));
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_3)
                .await;

            let dotrain_worsened =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x03", "50", "3");
            let (_, _) = deploy_order(&setup, dotrain_worsened).await;

            let taker = setup.local_evm.signer_wallets[1].default_signer().address();
            let taker_balance = U256::from(10).pow(U256::from(22));
            let token1_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token1)
                .unwrap();

            token1_contract
                .transfer(taker, taker_balance)
                .from(setup.owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            token1_contract
                .approve(setup.orderbook, taker_balance)
                .from(taker)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            let tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(result.calldata.to_vec())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let token2_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token2)
                .unwrap();

            let taker_token2_before: U256 = token2_contract.balanceOf(taker).call().await.unwrap();

            let tx_result = setup.local_evm.send_transaction(tx).await;
            assert!(
                tx_result.is_ok(),
                "Transaction with original calldata should succeed, got: {:?}",
                tx_result
            );

            let taker_token2_after: U256 = token2_contract.balanceOf(taker).call().await.unwrap();

            let received = taker_token2_after - taker_token2_before;
            let expected_from_cheap_only =
                Float::from_fixed_decimal(U256::from(50) * U256::from(10).pow(U256::from(18)), 18)
                    .unwrap();

            let received_float = Float::from_fixed_decimal(received, 18).unwrap();
            assert!(
                received_float.lte(expected_from_cheap_only).unwrap(),
                "Should only receive from cheap order since expensive order's vault was emptied, got: {:?}",
                received_float.format()
            );

            assert!(
                received > U256::ZERO,
                "Should have received tokens from cheap order"
            );

            let exact_config = TakeOrdersConfigV4 {
                minimumInput: original_config.maximumInput,
                maximumInput: original_config.maximumInput,
                maximumIORatio: original_config.maximumIORatio,
                orders: original_config.orders.clone(),
                data: original_config.data.clone(),
            };

            let exact_calldata_bytes = takeOrders3Call {
                config: exact_config,
            }
            .abi_encode();

            let exact_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(exact_calldata_bytes)
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let exact_call_result = setup.local_evm.call(exact_tx).await;
            assert!(
                exact_call_result.is_err(),
                "Exact mode should revert when simulated buy cannot be achieved after vault emptied, got: {:?}",
                exact_call_result
            );
        }

        #[tokio::test]
        async fn test_cross_orderbook_selection_picks_best_book() {
            let setup = setup_multi_orderbook_test().await;
            let sg_server = MockServer::start_async().await;

            assert_ne!(
                setup.orderbook_a, setup.orderbook_b,
                "Orderbook addresses should be different"
            );

            let vault_id_a = B256::from(U256::from(1u64));
            let vault_id_b = B256::from(U256::from(2u64));

            let deposit_amount = U256::from(10).pow(U256::from(22));
            deposit_to_orderbook(
                &setup,
                setup.orderbook_a,
                setup.token2,
                deposit_amount,
                vault_id_a,
            )
            .await;
            deposit_to_orderbook(
                &setup,
                setup.orderbook_b,
                setup.token2,
                deposit_amount,
                vault_id_b,
            )
            .await;

            let dotrain_a =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_a, "0x01", "5", "2");
            let (order_bytes_a, order_hash_a, _order_v4_a) =
                deploy_order_to_orderbook(&setup, setup.orderbook_a, dotrain_a).await;

            let dotrain_b =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_b, "0x02", "8", "2");
            let (order_bytes_b, order_hash_b, order_v4_b) =
                deploy_order_to_orderbook(&setup, setup.orderbook_b, dotrain_b).await;

            let vault_a_input =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token1_sg);
            let vault_a_output =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token2_sg);
            let vault_b_input =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token1_sg);
            let vault_b_output =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token2_sg);

            let sg_order_a = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_a,
                &order_bytes_a,
                order_hash_a,
                vec![vault_a_input],
                vec![vault_a_output],
            );
            let sg_order_b = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_b,
                &order_bytes_b,
                order_hash_b,
                vec![vault_b_input],
                vec![vault_b_output],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_a, sg_order_b]
                    }
                }));
            });

            let yaml = get_multi_orderbook_yaml(
                123,
                &setup.local_evm.url(),
                &sg_server.url("/sg"),
                &setup.orderbook_a.to_string(),
                &setup.orderbook_b.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let sell_budget = "100";
            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    sell_budget.to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should succeed with orders from multiple orderbooks");

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert_eq!(
                result.orderbook, setup.orderbook_b,
                "Should select orderbook B (max_output=8 > max_output=5)"
            );

            assert!(
                !config.orders.is_empty(),
                "Should have at least one order from the winning orderbook"
            );

            for config_item in &config.orders {
                let config_order = &config_item.order;
                assert_eq!(
                    config_order.owner, order_v4_b.owner,
                    "All orders should be from orderbook B"
                );
                assert_eq!(
                    config_order.evaluable.bytecode, order_v4_b.evaluable.bytecode,
                    "All order bytecodes should match orderbook B's order"
                );
            }

            let expected_ratio = Float::parse("2".to_string()).unwrap();
            assert!(
                result.prices[0].eq(expected_ratio).unwrap(),
                "Price should be 2 (orderbook B's ratio)"
            );

            let tolerance = Float::parse("0.0001".to_string()).unwrap();
            let diff = if result.effective_price.gt(expected_ratio).unwrap() {
                result.effective_price.sub(expected_ratio).unwrap()
            } else {
                expected_ratio.sub(result.effective_price).unwrap()
            };
            assert!(
                diff.lte(tolerance).unwrap(),
                "Effective price should be ~2 (sell/buy ratio), got: {:?}",
                result.effective_price.format()
            );
        }

        #[tokio::test]
        async fn test_cross_orderbook_selection_flips_when_economics_flip() {
            let setup = setup_multi_orderbook_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id_a = B256::from(U256::from(1u64));
            let vault_id_b = B256::from(U256::from(2u64));

            let deposit_amount = U256::from(10).pow(U256::from(22));
            deposit_to_orderbook(
                &setup,
                setup.orderbook_a,
                setup.token2,
                deposit_amount,
                vault_id_a,
            )
            .await;
            deposit_to_orderbook(
                &setup,
                setup.orderbook_b,
                setup.token2,
                deposit_amount,
                vault_id_b,
            )
            .await;

            let dotrain_a =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_a, "0x01", "10", "2");
            let (order_bytes_a, order_hash_a, order_v4_a) =
                deploy_order_to_orderbook(&setup, setup.orderbook_a, dotrain_a).await;

            let dotrain_b =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_b, "0x02", "3", "2");
            let (order_bytes_b, order_hash_b, _order_v4_b) =
                deploy_order_to_orderbook(&setup, setup.orderbook_b, dotrain_b).await;

            let vault_a_input =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token1_sg);
            let vault_a_output =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token2_sg);
            let vault_b_input =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token1_sg);
            let vault_b_output =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token2_sg);

            let sg_order_a = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_a,
                &order_bytes_a,
                order_hash_a,
                vec![vault_a_input],
                vec![vault_a_output],
            );
            let sg_order_b = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_b,
                &order_bytes_b,
                order_hash_b,
                vec![vault_b_input],
                vec![vault_b_output],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_a, sg_order_b]
                    }
                }));
            });

            let yaml = get_multi_orderbook_yaml(
                123,
                &setup.local_evm.url(),
                &sg_server.url("/sg"),
                &setup.orderbook_a.to_string(),
                &setup.orderbook_b.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let sell_budget = "100";
            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    sell_budget.to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should succeed with flipped economics");

            assert_eq!(
                result.orderbook, setup.orderbook_a,
                "Should select orderbook A (max_output=10 > max_output=3)"
            );

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert!(
                !config.orders.is_empty(),
                "Should have at least one order from the winning orderbook"
            );

            for config_item in &config.orders {
                let config_order = &config_item.order;
                assert_eq!(
                    config_order.owner, order_v4_a.owner,
                    "All orders should be from orderbook A"
                );
                assert_eq!(
                    config_order.evaluable.bytecode, order_v4_a.evaluable.bytecode,
                    "All order bytecodes should match orderbook A's order"
                );
            }

            let actual_max_input = Float::from_raw(config.maximumInput);
            let min_expected = Float::parse("10".to_string()).unwrap();
            assert!(
                actual_max_input.gte(min_expected).unwrap(),
                "maximumInput should be at least 10 (orderbook A's max_output), got: {:?}",
                actual_max_input.format()
            );
        }

        #[tokio::test]
        async fn test_cross_orderbook_economic_selection_prefers_best_yield() {
            let setup = setup_multi_orderbook_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id_a = B256::from(U256::from(1u64));
            let vault_id_b = B256::from(U256::from(2u64));

            let deposit_amount = U256::from(10).pow(U256::from(22));
            deposit_to_orderbook(
                &setup,
                setup.orderbook_a,
                setup.token2,
                deposit_amount,
                vault_id_a,
            )
            .await;
            deposit_to_orderbook(
                &setup,
                setup.orderbook_b,
                setup.token2,
                deposit_amount,
                vault_id_b,
            )
            .await;

            let dotrain_a =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_a, "0x01", "5", "1");
            let (order_bytes_a, order_hash_a, order_v4_a) =
                deploy_order_to_orderbook(&setup, setup.orderbook_a, dotrain_a).await;

            let dotrain_b =
                create_dotrain_config_for_orderbook(&setup, setup.orderbook_b, "0x02", "8", "1.5");
            let (order_bytes_b, order_hash_b, _order_v4_b) =
                deploy_order_to_orderbook(&setup, setup.orderbook_b, dotrain_b).await;

            let vault_a_input =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token1_sg);
            let vault_a_output =
                create_vault_for_orderbook(vault_id_a, &setup, setup.orderbook_a, &setup.token2_sg);
            let vault_b_input =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token1_sg);
            let vault_b_output =
                create_vault_for_orderbook(vault_id_b, &setup, setup.orderbook_b, &setup.token2_sg);

            let sg_order_a = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_a,
                &order_bytes_a,
                order_hash_a,
                vec![vault_a_input],
                vec![vault_a_output],
            );
            let sg_order_b = create_sg_order_json_with_orderbook(
                &setup,
                setup.orderbook_b,
                &order_bytes_b,
                order_hash_b,
                vec![vault_b_input],
                vec![vault_b_output],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_a, sg_order_b]
                    }
                }));
            });

            let yaml = get_multi_orderbook_yaml(
                123,
                &setup.local_evm.url(),
                &sg_server.url("/sg"),
                &setup.orderbook_a.to_string(),
                &setup.orderbook_b.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let sell_budget = "5";
            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    sell_budget.to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should succeed with orders from multiple orderbooks");

            assert_eq!(
                result.orderbook, setup.orderbook_a,
                "Should select orderbook A (5 sell yields ~5 buy at ratio 1.0) over B (5 sell yields ~3.33 buy at ratio 1.5)"
            );

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert!(
                !config.orders.is_empty(),
                "Should have at least one order from the winning orderbook"
            );

            for config_item in &config.orders {
                let config_order = &config_item.order;
                assert_eq!(
                    config_order.owner, order_v4_a.owner,
                    "All orders should be from orderbook A"
                );
                assert_eq!(
                    config_order.evaluable.bytecode, order_v4_a.evaluable.bytecode,
                    "All order bytecodes should match orderbook A's order"
                );
            }

            assert_eq!(
                result.prices.len(),
                1,
                "Should have exactly one price (from orderbook A only)"
            );
            let expected_ratio = Float::parse("1".to_string()).unwrap();
            assert!(
                result.prices[0].eq(expected_ratio).unwrap(),
                "Price should be 1.0 (orderbook A's ratio), got: {:?}",
                result.prices[0].format()
            );

            let tolerance = Float::parse("0.0001".to_string()).unwrap();
            let diff = if result.effective_price.gt(expected_ratio).unwrap() {
                result.effective_price.sub(expected_ratio).unwrap()
            } else {
                expected_ratio.sub(result.effective_price).unwrap()
            };
            assert!(
                diff.lte(tolerance).unwrap(),
                "Effective price should be ~1.0 (total_sell/total_buy), got: {:?}",
                result.effective_price.format()
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_invalid_address_returns_from_hex_error() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:0/unused_sg1",
                    "http://localhost:0/unused_sg2",
                    "http://localhost:0/unused_rpc1",
                    "http://localhost:0/unused_rpc2",
                )],
                None,
            )
            .unwrap();

            let res = client
                .get_take_orders_calldata(
                    1,
                    "not-an-address".to_string(),
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                    "1".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::FromHexError(_))),
                "Expected FromHexError for invalid sellToken address, got: {:?}",
                res
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_invalid_float_returns_float_error() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:0/unused_sg1",
                    "http://localhost:0/unused_sg2",
                    "http://localhost:0/unused_rpc1",
                    "http://localhost:0/unused_rpc2",
                )],
                None,
            )
            .unwrap();

            let res = client
                .get_take_orders_calldata(
                    1,
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                    "not-a-float".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::Float(_))),
                "Expected Float error for invalid sellAmount, got: {:?}",
                res
            );
        }

        #[test]
        fn test_select_best_orderbook_tiebreak_identical_totals_prefers_lower_address() {
            let ob_higher = Address::from([0x22u8; 20]);
            let ob_lower = Address::from([0x11u8; 20]);

            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("1".to_string()).unwrap();

            let higher_candidate = make_candidate(ob_higher, max_output, ratio);
            let lower_candidate = make_candidate(ob_lower, max_output, ratio);

            let sell_budget = Float::parse("100".to_string()).unwrap();

            for _ in 0..20 {
                let candidates = vec![higher_candidate.clone(), lower_candidate.clone()];
                let result = select_best_orderbook_simulation(candidates, sell_budget);
                assert!(result.is_ok());
                let (winner, sim) = result.unwrap();

                assert_eq!(
                    winner, ob_lower,
                    "Tie-break rule: when total_buy amounts and worst prices are equal, \
                     prefer the lower orderbook address (0x{:x} < 0x{:x})",
                    ob_lower, ob_higher
                );
                assert_eq!(sim.legs.len(), 1);
                assert_eq!(sim.legs[0].candidate.orderbook, ob_lower);
            }
        }

        #[test]
        fn test_select_best_orderbook_tiebreak_identical_totals_prefers_lower_worst_price() {
            let ob_better_price = Address::from([0x22u8; 20]);
            let ob_worse_price = Address::from([0x11u8; 20]);

            let max_output = Float::parse("10".to_string()).unwrap();
            let better_ratio = Float::parse("0.9".to_string()).unwrap();
            let worse_ratio = Float::parse("1.1".to_string()).unwrap();

            let better_candidate = make_candidate(ob_better_price, max_output, better_ratio);
            let worse_candidate = make_candidate(ob_worse_price, max_output, worse_ratio);

            let sell_budget = Float::parse("100".to_string()).unwrap();

            for _ in 0..20 {
                let candidates = vec![worse_candidate.clone(), better_candidate.clone()];
                let result = select_best_orderbook_simulation(candidates, sell_budget);
                assert!(result.is_ok());
                let (winner, sim) = result.unwrap();

                assert_eq!(
                    winner, ob_better_price,
                    "Tie-break rule: when total_buy amounts are equal, \
                     prefer the orderbook with the lower worst price (ratio 0.9 < 1.1)"
                );
                assert_eq!(sim.legs.len(), 1);
                assert_eq!(sim.legs[0].candidate.orderbook, ob_better_price);
            }
        }

        #[tokio::test]
        async fn test_prices_sorted_best_to_worst_matching_config_orders() {
            let setup = base_setup_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id_1 = B256::from(U256::from(1u64));
            let vault_id_2 = B256::from(U256::from(2u64));

            let amount = standard_deposit_amount();
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_1)
                .await;
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id_2)
                .await;

            let dotrain_cheap =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x01", "100", "1");
            let dotrain_expensive =
                create_dotrain_config_with_vault_and_ratio(&setup, "0x02", "100", "2");

            let (order_bytes_cheap, order_hash_cheap) = deploy_order(&setup, dotrain_cheap).await;
            let (order_bytes_expensive, order_hash_expensive) =
                deploy_order(&setup, dotrain_expensive).await;

            let vault1 = create_vault(vault_id_1, &setup, &setup.token2_sg);
            let vault2 = create_vault(vault_id_2, &setup, &setup.token2_sg);
            let input_vault = create_vault(vault_id_1, &setup, &setup.token1_sg);

            let sg_order_cheap = create_sg_order_json(
                &setup,
                &order_bytes_cheap,
                order_hash_cheap,
                vec![input_vault.clone()],
                vec![vault1.clone()],
            );
            let sg_order_expensive = create_sg_order_json(
                &setup,
                &order_bytes_expensive,
                order_hash_expensive,
                vec![input_vault.clone()],
                vec![vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [sg_order_expensive.clone(), sg_order_cheap.clone()]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "200".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should build calldata with both orders");

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert_eq!(result.prices.len(), 2, "Should have 2 prices for 2 orders");
            assert_eq!(config.orders.len(), 2, "Should have 2 orders in config");

            let cheap_price = Float::parse("1".to_string()).unwrap();
            let expensive_price = Float::parse("2".to_string()).unwrap();

            assert!(
                result.prices[0].eq(cheap_price).unwrap(),
                "First price should be cheap (1), got: {:?}",
                result.prices[0].format()
            );
            assert!(
                result.prices[1].eq(expensive_price).unwrap(),
                "Second price should be expensive (2), got: {:?}",
                result.prices[1].format()
            );

            assert!(
                result.prices[0].lt(result.prices[1]).unwrap(),
                "Prices should be sorted best (lowest) to worst: {:?} < {:?}",
                result.prices[0].format(),
                result.prices[1].format()
            );

            let first_order_hash = B256::from(keccak256(config.orders[0].order.abi_encode()));
            let second_order_hash = B256::from(keccak256(config.orders[1].order.abi_encode()));

            assert_eq!(
                first_order_hash, order_hash_cheap,
                "First order in config should be the cheap order (ratio=1)"
            );
            assert_eq!(
                second_order_hash, order_hash_expensive,
                "Second order in config should be the expensive order (ratio=2)"
            );
        }
    }
}
