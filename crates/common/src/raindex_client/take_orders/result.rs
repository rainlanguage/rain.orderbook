use crate::raindex_client::RaindexError;
use crate::take_orders::BuiltTakeOrdersConfig;
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use serde::{Deserialize, Serialize};
use std::ops::Div;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct TakeOrdersCalldataResult {
    orderbook: Address,
    calldata: Bytes,
    effective_price: Float,
    prices: Vec<Float>,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl TakeOrdersCalldataResult {
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn calldata(&self) -> String {
        self.calldata.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn effective_price(&self) -> Float {
        self.effective_price
    }
    #[wasm_bindgen(getter)]
    pub fn prices(&self) -> Vec<Float> {
        self.prices.clone()
    }
}
#[cfg(not(target_family = "wasm"))]
impl TakeOrdersCalldataResult {
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
    pub fn calldata(&self) -> Bytes {
        self.calldata.clone()
    }
    pub fn effective_price(&self) -> Float {
        self.effective_price
    }
    pub fn prices(&self) -> Vec<Float> {
        self.prices
    }
}

pub(crate) fn build_calldata_result(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::take_orders::selection::select_best_orderbook_simulation;
    use crate::take_orders::{build_take_orders_config_from_buy_simulation, MinReceiveMode};
    use crate::test_helpers::candidates::make_candidate;
    use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;

    fn high_price_cap() -> Float {
        Float::parse("1000000".to_string()).unwrap()
    }

    #[test]
    fn test_build_calldata_result_produces_valid_calldata() {
        let ob = Address::from([0x11u8; 20]);
        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob, max_output, ratio);
        let candidates = vec![candidate];
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = high_price_cap();

        let (_, sim) = select_best_orderbook_simulation(candidates, buy_target, price_cap).unwrap();
        let built = build_take_orders_config_from_buy_simulation(
            sim,
            buy_target,
            price_cap,
            MinReceiveMode::Partial,
        )
        .unwrap()
        .unwrap();

        let result = build_calldata_result(ob, built);

        assert!(result.is_ok());
        let calldata_result = result.unwrap();
        assert_eq!(calldata_result.orderbook, ob);
        assert!(!calldata_result.calldata.is_empty());
        assert!(!calldata_result.prices.is_empty());

        let decoded =
            takeOrders3Call::abi_decode(&calldata_result.calldata).expect("Should decode calldata");
        assert!(!decoded.config.orders.is_empty());
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

        let (_, sim) = select_best_orderbook_simulation(candidates, buy_target, price_cap).unwrap();
        let built = build_take_orders_config_from_buy_simulation(
            sim,
            buy_target,
            price_cap,
            MinReceiveMode::Partial,
        )
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
        let buy_target = Float::parse("10".to_string()).unwrap();
        let price_cap = high_price_cap();

        let (_, sim) = select_best_orderbook_simulation(candidates, buy_target, price_cap).unwrap();
        let leg_count = sim.legs.len();
        let built = build_take_orders_config_from_buy_simulation(
            sim,
            buy_target,
            price_cap,
            MinReceiveMode::Partial,
        )
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
}
