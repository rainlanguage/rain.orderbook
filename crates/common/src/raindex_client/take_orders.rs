use super::orders::{GetOrdersFilters, GetOrdersTokenFilter};
use super::*;
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_order_candidates_for_pair, build_take_orders_config_from_sell_simulation,
    simulate_sell_over_candidates, MinReceiveMode, SimulatedSellResult, TakeOrderCandidate,
};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use std::collections::HashMap;
use std::ops::Div;
use std::str::FromStr;

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
        let sell_token_addr = Address::from_str(&sell_token)?;
        let buy_token_addr = Address::from_str(&buy_token)?;
        let sell_amount_float = Float::parse(sell_amount)?;

        let filters = GetOrdersFilters {
            owners: vec![],
            active: Some(true),
            order_hash: None,
            tokens: Some(GetOrdersTokenFilter {
                inputs: Some(vec![sell_token_addr]),
                outputs: Some(vec![buy_token_addr]),
            }),
        };

        let orders = self
            .get_orders(Some(ChainIds(vec![chain_id])), Some(filters), None)
            .await?;

        if orders.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let rpc_urls = self.get_rpc_urls_for_chain(chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls)?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates = build_take_order_candidates_for_pair(
            &orders,
            sell_token_addr,
            buy_token_addr,
            Some(block_number),
            None,
        )
        .await?;

        if candidates.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let (best_orderbook, best_sim) =
            select_best_orderbook_simulation(candidates, sell_amount_float)?;

        let built = build_take_orders_config_from_sell_simulation(best_sim, min_receive_mode)?
            .ok_or(RaindexError::NoLiquidity)?;

        let calldata_bytes = takeOrders3Call {
            config: built.config,
        }
        .abi_encode();
        let calldata = Bytes::copy_from_slice(&calldata_bytes);

        let zero = Float::zero()?;
        let effective_price = if built.sim.total_buy_amount.gt(zero)? {
            built
                .sim
                .total_sell_amount
                .div(built.sim.total_buy_amount)?
        } else {
            zero
        };

        let prices: Vec<Float> = built
            .sim
            .legs
            .iter()
            .map(|leg| leg.candidate.ratio)
            .collect();

        Ok(TakeOrdersCalldataResult {
            orderbook: best_orderbook,
            calldata,
            effective_price,
            prices,
        })
    }
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
            Some((_, best_sim)) => sim.total_buy_amount.gt(best_sim.total_buy_amount)?,
        };

        if is_better {
            best_result = Some((orderbook, sim));
        }
    }

    best_result.ok_or(RaindexError::NoLiquidity)
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
        use super::super::select_best_orderbook_simulation;
        use crate::raindex_client::RaindexError;
        use crate::take_orders::TakeOrderCandidate;
        use alloy::primitives::{Address, U256};
        use rain_math_float::Float;
        use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};

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

        fn make_candidate(
            orderbook: Address,
            max_output: Float,
            ratio: Float,
        ) -> TakeOrderCandidate {
            TakeOrderCandidate {
                orderbook,
                order: make_basic_order(Address::from([4u8; 20]), Address::from([5u8; 20])),
                input_io_index: 0,
                output_io_index: 0,
                max_output,
                ratio,
            }
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
    }
}
