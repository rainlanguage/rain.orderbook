use super::simulation::SimulatedSellResult;
use crate::raindex_client::RaindexError;
use alloy::primitives::{Bytes, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{
    SignedContextV1, TakeOrderConfigV4, TakeOrdersConfigV4,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum MinReceiveMode {
    Partial,
    Exact,
}
impl_wasm_traits!(MinReceiveMode);

#[derive(Clone, Debug)]
pub struct BuiltTakeOrdersConfig {
    pub config: TakeOrdersConfigV4,
    pub sim: SimulatedSellResult,
}

pub fn build_take_orders_config_from_sell_simulation(
    sim: SimulatedSellResult,
    min_receive_mode: MinReceiveMode,
) -> Result<Option<BuiltTakeOrdersConfig>, RaindexError> {
    if sim.legs.is_empty() {
        return Ok(None);
    }

    let zero = Float::zero()?;

    let orders: Vec<TakeOrderConfigV4> = sim
        .legs
        .iter()
        .map(|leg| TakeOrderConfigV4 {
            order: leg.candidate.order.clone(),
            inputIOIndex: U256::from(leg.candidate.input_io_index),
            outputIOIndex: U256::from(leg.candidate.output_io_index),
            signedContext: vec![] as Vec<SignedContextV1>,
        })
        .collect();

    let maximum_input = sim.total_buy_amount.get_inner();

    let minimum_input = match min_receive_mode {
        MinReceiveMode::Partial => zero.get_inner(),
        MinReceiveMode::Exact => sim.total_buy_amount.get_inner(),
    };

    let mut worst_price = zero;
    for leg in &sim.legs {
        if leg.candidate.ratio.gt(worst_price)? {
            worst_price = leg.candidate.ratio;
        }
    }
    let maximum_io_ratio = worst_price.get_inner();

    let config = TakeOrdersConfigV4 {
        minimumInput: minimum_input,
        maximumInput: maximum_input,
        maximumIORatio: maximum_io_ratio,
        orders,
        data: Bytes::new(),
    };

    Ok(Some(BuiltTakeOrdersConfig { config, sim }))
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::take_orders::simulation::{simulate_sell_over_candidates, SelectedTakeOrderLeg};
    use crate::test_helpers::candidates::make_simulation_candidate;
    use rain_math_float::Float;

    #[test]
    fn test_build_config_empty_simulation() {
        let sim = SimulatedSellResult {
            legs: vec![],
            total_buy_amount: Float::zero().unwrap(),
            total_sell_amount: Float::zero().unwrap(),
        };

        let result =
            build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_build_config_partial_mode() {
        let f1_75 = Float::parse("1.75".to_string()).unwrap();
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_5, f1_75);
        let sim = SimulatedSellResult {
            legs: vec![SelectedTakeOrderLeg {
                candidate,
                buy_amount: f2_5,
                sell_amount: f2_5,
            }],
            total_buy_amount: f2_5,
            total_sell_amount: f2_5,
        };

        let result = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
            .unwrap()
            .unwrap();

        assert_eq!(
            result.config.minimumInput,
            Float::zero().unwrap().get_inner()
        );
        assert_eq!(result.config.maximumInput, f2_5.get_inner());
        assert_eq!(result.config.maximumIORatio, f1_75.get_inner());
        assert_eq!(result.config.orders.len(), 1);
    }

    #[test]
    fn test_build_config_exact_mode() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f2_75 = Float::parse("2.75".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_75, f1_25);
        let sim = SimulatedSellResult {
            legs: vec![SelectedTakeOrderLeg {
                candidate,
                buy_amount: f2_75,
                sell_amount: f2_75,
            }],
            total_buy_amount: f2_75,
            total_sell_amount: f2_75,
        };

        let result = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Exact)
            .unwrap()
            .unwrap();

        assert_eq!(result.config.minimumInput, f2_75.get_inner());
        assert_eq!(result.config.maximumInput, f2_75.get_inner());
        assert_eq!(result.config.maximumIORatio, f1_25.get_inner());
    }

    #[test]
    fn test_build_config_worst_price_from_multiple_legs() {
        let f1_5 = Float::parse("1.5".to_string()).unwrap();
        let f2_75 = Float::parse("2.75".to_string()).unwrap();
        let f3_25 = Float::parse("3.25".to_string()).unwrap();
        let cheap_candidate = make_simulation_candidate(f1_5, f1_5);
        let expensive_candidate = make_simulation_candidate(f1_5, f2_75);
        let sim = SimulatedSellResult {
            legs: vec![
                SelectedTakeOrderLeg {
                    candidate: cheap_candidate,
                    buy_amount: f1_5,
                    sell_amount: f1_5,
                },
                SelectedTakeOrderLeg {
                    candidate: expensive_candidate,
                    buy_amount: f1_5,
                    sell_amount: f2_75,
                },
            ],
            total_buy_amount: f3_25,
            total_sell_amount: Float::parse("4.25".to_string()).unwrap(),
        };

        let result = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
            .unwrap()
            .unwrap();

        assert_eq!(
            result.config.maximumIORatio,
            f2_75.get_inner(),
            "Should use worst (highest) price"
        );
        assert_eq!(result.config.orders.len(), 2);
    }

    #[test]
    fn test_build_config_preserves_order() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let f3_75 = Float::parse("3.75".to_string()).unwrap();
        let candidate1 = make_simulation_candidate(f1_25, f1_25);
        let candidate2 = make_simulation_candidate(f1_25, f2_5);
        let sim = SimulatedSellResult {
            legs: vec![
                SelectedTakeOrderLeg {
                    candidate: candidate1.clone(),
                    buy_amount: f1_25,
                    sell_amount: f1_25,
                },
                SelectedTakeOrderLeg {
                    candidate: candidate2.clone(),
                    buy_amount: f1_25,
                    sell_amount: f2_5,
                },
            ],
            total_buy_amount: f2_5,
            total_sell_amount: f3_75,
        };

        let result = build_take_orders_config_from_sell_simulation(sim, MinReceiveMode::Partial)
            .unwrap()
            .unwrap();

        assert_eq!(result.config.orders.len(), 2);
        assert_eq!(
            result.config.orders[0].inputIOIndex,
            alloy::primitives::U256::from(0)
        );
        assert_eq!(
            result.config.orders[1].inputIOIndex,
            alloy::primitives::U256::from(0)
        );
    }

    #[test]
    fn test_simulate_multi_leg_partial_fill_second_leg_with_config() {
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let ratio_3 = Float::parse("3".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let candidate_cheap = make_simulation_candidate(max_output, ratio_1);
        let candidate_mid = make_simulation_candidate(max_output, ratio_2);
        let candidate_expensive = make_simulation_candidate(max_output, ratio_3);

        let candidates = vec![candidate_expensive, candidate_mid, candidate_cheap];
        let sell_budget = Float::parse("150".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        let expected_total_buy = Float::parse("125".to_string()).unwrap();

        let built = build_take_orders_config_from_sell_simulation(result, MinReceiveMode::Partial)
            .unwrap()
            .unwrap();

        assert_eq!(
            built.config.maximumIORatio,
            ratio_2.get_inner(),
            "maximumIORatio should be ratio_2 (worst among used legs)"
        );
        assert_eq!(
            built.config.orders.len(),
            2,
            "orders length should match legs length"
        );
        assert!(
            Float::from_raw(built.config.maximumInput)
                .eq(expected_total_buy)
                .unwrap(),
            "maximumInput should equal total_buy_amount"
        );
    }
}
