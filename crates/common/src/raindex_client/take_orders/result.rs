use crate::raindex_client::RaindexError;
use crate::take_orders::{BuiltTakeOrdersConfig, ParsedTakeOrdersMode};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV6::takeOrders4Call;
use serde::{Deserialize, Serialize};
use std::ops::{Div, Mul};
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct TakeOrdersCalldataResult {
    orderbook: Address,
    calldata: Bytes,
    effective_price: Float,
    prices: Vec<Float>,
    expected_sell: Float,
    max_sell_cap: Float,
}

impl TakeOrdersCalldataResult {
    pub(crate) fn new(
        orderbook: Address,
        calldata: Bytes,
        effective_price: Float,
        prices: Vec<Float>,
        expected_sell: Float,
        max_sell_cap: Float,
    ) -> Self {
        Self {
            orderbook,
            calldata,
            effective_price,
            prices,
            expected_sell,
            max_sell_cap,
        }
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl TakeOrdersCalldataResult {
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn calldata(&self) -> String {
        self.calldata.to_string()
    }
    #[wasm_bindgen(getter = effectivePrice)]
    pub fn effective_price(&self) -> Float {
        self.effective_price
    }
    #[wasm_bindgen(getter)]
    pub fn prices(&self) -> Vec<Float> {
        self.prices.clone()
    }
    #[wasm_bindgen(getter = expectedSell)]
    pub fn expected_sell(&self) -> Float {
        self.expected_sell
    }
    #[wasm_bindgen(getter = maxSellCap)]
    pub fn max_sell_cap(&self) -> Float {
        self.max_sell_cap
    }
}

#[cfg(not(target_family = "wasm"))]
impl TakeOrdersCalldataResult {
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
    pub fn calldata(&self) -> &Bytes {
        &self.calldata
    }
    pub fn effective_price(&self) -> Float {
        self.effective_price
    }
    pub fn prices(&self) -> &Vec<Float> {
        &self.prices
    }
    pub fn expected_sell(&self) -> Float {
        self.expected_sell
    }
    pub fn max_sell_cap(&self) -> Float {
        self.max_sell_cap
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct TakeOrderEstimate {
    expected_spend: Float,
    expected_receive: Float,
    is_partial: bool,
}

impl TakeOrderEstimate {
    pub(crate) fn new(expected_spend: Float, expected_receive: Float, is_partial: bool) -> Self {
        Self {
            expected_spend,
            expected_receive,
            is_partial,
        }
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl TakeOrderEstimate {
    #[wasm_bindgen(getter = expectedSpend)]
    pub fn expected_spend(&self) -> Float {
        self.expected_spend
    }
    #[wasm_bindgen(getter = expectedReceive)]
    pub fn expected_receive(&self) -> Float {
        self.expected_receive
    }
    #[wasm_bindgen(getter = isPartial)]
    pub fn is_partial(&self) -> bool {
        self.is_partial
    }
}

#[cfg(not(target_family = "wasm"))]
impl TakeOrderEstimate {
    pub fn expected_spend(&self) -> Float {
        self.expected_spend
    }
    pub fn expected_receive(&self) -> Float {
        self.expected_receive
    }
    pub fn is_partial(&self) -> bool {
        self.is_partial
    }
}

pub(crate) fn build_calldata_result(
    orderbook: Address,
    built_config: BuiltTakeOrdersConfig,
    mode: ParsedTakeOrdersMode,
    price_cap: Float,
) -> Result<TakeOrdersCalldataResult, RaindexError> {
    let calldata_bytes = takeOrders4Call {
        config: built_config.config,
    }
    .abi_encode();
    let calldata = Bytes::copy_from_slice(&calldata_bytes);

    let zero = Float::zero()?;
    let effective_price = if built_config.sim.total_output.gt(zero)? {
        built_config
            .sim
            .total_input
            .div(built_config.sim.total_output)?
    } else {
        zero
    };

    let prices: Vec<Float> = built_config
        .sim
        .legs
        .iter()
        .map(|leg| leg.candidate.ratio)
        .collect();

    let expected_sell = built_config.sim.total_input;
    let max_sell_cap = if mode.is_buy_mode() {
        mode.target_amount().mul(price_cap)?
    } else {
        mode.target_amount()
    };

    Ok(TakeOrdersCalldataResult::new(
        orderbook,
        calldata,
        effective_price,
        prices,
        expected_sell,
        max_sell_cap,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::take_orders::selection::select_best_orderbook_simulation;
    use crate::take_orders::build_take_orders_config_from_simulation;
    use crate::test_helpers::candidates::make_candidate;
    use rain_orderbook_bindings::IOrderBookV6::takeOrders4Call;

    fn high_price_cap() -> Float {
        Float::parse("1000000".to_string()).unwrap()
    }

    fn buy_up_to(amount: Float) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode: crate::take_orders::TakeOrdersMode::BuyUpTo,
            amount,
        }
    }

    fn spend_up_to(amount: Float) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode: crate::take_orders::TakeOrdersMode::SpendUpTo,
            amount,
        }
    }

    #[test]
    fn test_build_calldata_result_produces_valid_calldata_buy_mode() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = high_price_cap();
        let mode = buy_up_to(buy_target);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap);

        assert!(result.is_ok());
        let calldata_result = result.unwrap();
        assert_eq!(calldata_result.orderbook, ob);
        assert!(!calldata_result.calldata.is_empty());
        assert!(!calldata_result.prices.is_empty());

        let decoded =
            takeOrders4Call::abi_decode(&calldata_result.calldata).expect("Should decode calldata");
        assert!(!decoded.config.orders.is_empty());
        assert!(
            decoded.config.IOIsInput,
            "IOIsInput should be true for buy mode (taker output constraints)"
        );
    }

    #[test]
    fn test_build_calldata_result_produces_valid_calldata_spend_mode() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let spend_budget = Float::parse("20".to_string()).unwrap();
        let price_cap = high_price_cap();
        let mode = spend_up_to(spend_budget);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap);

        assert!(result.is_ok());
        let calldata_result = result.unwrap();
        assert_eq!(calldata_result.orderbook, ob);
        assert!(!calldata_result.calldata.is_empty());
        assert!(!calldata_result.prices.is_empty());

        let decoded =
            takeOrders4Call::abi_decode(&calldata_result.calldata).expect("Should decode calldata");
        assert!(!decoded.config.orders.is_empty());
        assert!(
            !decoded.config.IOIsInput,
            "IOIsInput should be false for spend mode (taker input constraints)"
        );
    }

    #[test]
    fn test_build_calldata_result_effective_price_calculation() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = high_price_cap();
        let mode = buy_up_to(buy_target);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap).unwrap();

        let zero = Float::zero().unwrap();
        assert!(
            result.effective_price().gt(zero).unwrap(),
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
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = high_price_cap();
        let mode = buy_up_to(buy_target);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let leg_count = sim.legs.len();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap).unwrap();

        assert_eq!(
            result.prices().len(),
            leg_count,
            "Number of prices should match number of legs"
        );
        assert!(
            result.prices()[0].eq(ratio).unwrap(),
            "Price should match the candidate ratio"
        );
    }

    #[test]
    fn test_build_calldata_result_expected_sell_and_max_sell_cap_buy_mode() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = Float::parse("3".to_string()).unwrap();
        let mode = buy_up_to(buy_target);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap).unwrap();

        let expected_sell = Float::parse("20".to_string()).unwrap();
        let expected_max_sell_cap = Float::parse("30".to_string()).unwrap();

        assert!(
            result.expected_sell().eq(expected_sell).unwrap(),
            "expected_sell should be output * ratio = 10 * 2 = 20"
        );
        assert!(
            result.max_sell_cap().eq(expected_max_sell_cap).unwrap(),
            "max_sell_cap should be buy_target * price_cap = 10 * 3 = 30"
        );
    }

    #[test]
    fn test_build_calldata_result_expected_sell_and_max_sell_cap_spend_mode() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let spend_budget = Float::parse("20".to_string()).unwrap();
        let price_cap = Float::parse("3".to_string()).unwrap();
        let mode = spend_up_to(spend_budget);

        let (_, sim) = select_best_orderbook_simulation(candidates, mode, price_cap).unwrap();
        let built = build_take_orders_config_from_simulation(sim, mode, price_cap)
            .unwrap()
            .unwrap();

        let result = build_calldata_result(ob, built, mode, price_cap).unwrap();

        let expected_sell = Float::parse("20".to_string()).unwrap();
        let expected_max_sell_cap = Float::parse("20".to_string()).unwrap();

        assert!(
            result.expected_sell().eq(expected_sell).unwrap(),
            "expected_sell should equal spend_budget = 20"
        );
        assert!(
            result.max_sell_cap().eq(expected_max_sell_cap).unwrap(),
            "max_sell_cap in spend mode should equal spend_budget = 20"
        );
    }
}
