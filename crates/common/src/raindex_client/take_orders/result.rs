use crate::raindex_client::RaindexError;
use crate::take_orders::{build_approval_calldata, BuiltTakeOrdersConfig, ParsedTakeOrdersMode};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV6::takeOrders4Call;
use serde::{Deserialize, Serialize};
use std::ops::{Div, Mul};
use wasm_bindgen_utils::impl_wasm_traits;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalInfo {
    #[tsify(type = "Address")]
    pub token: Address,
    #[tsify(type = "Address")]
    pub spender: Address,
    #[tsify(type = "Hex")]
    pub amount: Float,
    pub formatted_amount: String,
    #[tsify(type = "Hex")]
    pub calldata: Bytes,
}
impl_wasm_traits!(ApprovalInfo);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersInfo {
    #[tsify(type = "Address")]
    pub orderbook: Address,
    #[tsify(type = "Hex")]
    pub calldata: Bytes,
    #[tsify(type = "Hex")]
    pub effective_price: Float,
    #[tsify(type = "Hex[]")]
    pub prices: Vec<Float>,
    #[tsify(type = "Hex")]
    pub expected_sell: Float,
    #[tsify(type = "Hex")]
    pub max_sell_cap: Float,
}
impl_wasm_traits!(TakeOrdersInfo);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum TakeOrdersCalldataResult {
    NeedsApproval(ApprovalInfo),
    Ready(TakeOrdersInfo),
}
impl_wasm_traits!(TakeOrdersCalldataResult);

pub(crate) fn build_approval_result(
    token: Address,
    spender: Address,
    amount: Float,
    decimals: u8,
) -> Result<TakeOrdersCalldataResult, RaindexError> {
    let amount_u256 = amount.to_fixed_decimal(decimals)?;
    let calldata = build_approval_calldata(spender, amount_u256);
    let formatted_amount = amount.format().unwrap_or_default();
    Ok(TakeOrdersCalldataResult::NeedsApproval(ApprovalInfo {
        token,
        spender,
        amount,
        formatted_amount,
        calldata,
    }))
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

    Ok(TakeOrdersCalldataResult::Ready(TakeOrdersInfo {
        orderbook,
        calldata,
        effective_price,
        prices,
        expected_sell,
        max_sell_cap,
    }))
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result.unwrap() else {
            panic!("Expected Ready variant");
        };
        assert_eq!(take_orders_info.orderbook, ob);
        assert!(!take_orders_info.calldata.is_empty());
        assert!(!take_orders_info.prices.is_empty());

        let decoded = takeOrders4Call::abi_decode(&take_orders_info.calldata)
            .expect("Should decode calldata");
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result.unwrap() else {
            panic!("Expected Ready variant");
        };
        assert_eq!(take_orders_info.orderbook, ob);
        assert!(!take_orders_info.calldata.is_empty());
        assert!(!take_orders_info.prices.is_empty());

        let decoded = takeOrders4Call::abi_decode(&take_orders_info.calldata)
            .expect("Should decode calldata");
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result else {
            panic!("Expected Ready variant");
        };

        let zero = Float::zero().unwrap();
        assert!(
            take_orders_info.effective_price.gt(zero).unwrap(),
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result else {
            panic!("Expected Ready variant");
        };

        assert_eq!(
            take_orders_info.prices.len(),
            leg_count,
            "Number of prices should match number of legs"
        );
        assert!(
            take_orders_info.prices[0].eq(ratio).unwrap(),
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result else {
            panic!("Expected Ready variant");
        };

        let expected_sell = Float::parse("20".to_string()).unwrap();
        let expected_max_sell_cap = Float::parse("30".to_string()).unwrap();

        assert!(
            take_orders_info.expected_sell.eq(expected_sell).unwrap(),
            "expected_sell should be output * ratio = 10 * 2 = 20"
        );
        assert!(
            take_orders_info
                .max_sell_cap
                .eq(expected_max_sell_cap)
                .unwrap(),
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
        let TakeOrdersCalldataResult::Ready(take_orders_info) = result else {
            panic!("Expected Ready variant");
        };

        let expected_sell = Float::parse("20".to_string()).unwrap();
        let expected_max_sell_cap = Float::parse("20".to_string()).unwrap();

        assert!(
            take_orders_info.expected_sell.eq(expected_sell).unwrap(),
            "expected_sell should equal spend_budget = 20"
        );
        assert!(
            take_orders_info
                .max_sell_cap
                .eq(expected_max_sell_cap)
                .unwrap(),
            "max_sell_cap in spend mode should equal spend_budget = 20"
        );
    }

    #[test]
    fn test_build_approval_result_produces_valid_approval_info() {
        let token = Address::from([0x22u8; 20]);
        let spender = Address::from([0x33u8; 20]);
        let amount = Float::parse("1000".to_string()).unwrap();
        let decimals = 18u8;

        let result = build_approval_result(token, spender, amount, decimals).unwrap();

        let TakeOrdersCalldataResult::NeedsApproval(approval_info) = result else {
            panic!("Expected NeedsApproval variant");
        };
        assert_eq!(approval_info.token, token);
        assert_eq!(approval_info.spender, spender);
        assert!(approval_info.amount.eq(amount).unwrap());
        assert_eq!(approval_info.formatted_amount, "1000");
        assert!(!approval_info.calldata.is_empty());
    }
}
