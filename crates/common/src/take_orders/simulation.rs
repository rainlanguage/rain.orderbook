use super::candidates::TakeOrderCandidate;
use super::price::cmp_float;
use crate::raindex_client::RaindexError;
use rain_math_float::Float;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Debug)]
pub struct SelectedTakeOrderLeg {
    pub candidate: TakeOrderCandidate,
    pub buy_amount: Float,
    pub sell_amount: Float,
}

#[derive(Clone, Debug)]
pub struct SimulatedSellResult {
    pub legs: Vec<SelectedTakeOrderLeg>,
    pub total_buy_amount: Float,
    pub total_sell_amount: Float,
}

fn sort_candidates_by_price(candidates: &mut [TakeOrderCandidate]) -> Result<(), RaindexError> {
    let comparison_error: RefCell<Option<RaindexError>> = RefCell::new(None);
    candidates.sort_by(|a, b| {
        if comparison_error.borrow().is_some() {
            return Ordering::Equal;
        }
        match cmp_float(&a.ratio, &b.ratio) {
            Ok(ord) => ord,
            Err(e) => {
                *comparison_error.borrow_mut() = Some(e);
                Ordering::Equal
            }
        }
    });
    if let Some(e) = comparison_error.into_inner() {
        return Err(e);
    }
    Ok(())
}

fn take_leg(
    candidate: TakeOrderCandidate,
    remaining_sell: Float,
) -> Result<Option<SelectedTakeOrderLeg>, RaindexError> {
    let zero = Float::zero()?;
    let price = candidate.ratio;
    let max_buy = candidate.max_output;
    let full_sell_cost = max_buy.mul(price)?;

    let (buy_amount, sell_amount) = if full_sell_cost.lte(remaining_sell)? {
        (max_buy, full_sell_cost)
    } else {
        let sell = remaining_sell;
        let buy = if price.gt(zero)? {
            sell.div(price)?
        } else {
            zero
        };
        (buy, sell)
    };

    if buy_amount.lte(zero)? {
        return Ok(None);
    }

    Ok(Some(SelectedTakeOrderLeg {
        candidate,
        buy_amount,
        sell_amount,
    }))
}

struct SimulationTotals {
    remaining_sell: Float,
    total_buy_amount: Float,
    total_sell_amount: Float,
}

fn apply_leg(
    leg: &SelectedTakeOrderLeg,
    totals: &mut SimulationTotals,
) -> Result<(), RaindexError> {
    totals.remaining_sell = totals.remaining_sell.sub(leg.sell_amount)?;
    totals.total_buy_amount = totals.total_buy_amount.add(leg.buy_amount)?;
    totals.total_sell_amount = totals.total_sell_amount.add(leg.sell_amount)?;
    Ok(())
}

pub fn simulate_sell_over_candidates(
    mut candidates: Vec<TakeOrderCandidate>,
    sell_budget: Float,
) -> Result<SimulatedSellResult, RaindexError> {
    sort_candidates_by_price(&mut candidates)?;

    let zero = Float::zero()?;
    let mut totals = SimulationTotals {
        remaining_sell: sell_budget,
        total_buy_amount: zero,
        total_sell_amount: zero,
    };
    let mut legs: Vec<SelectedTakeOrderLeg> = Vec::new();

    for candidate in candidates {
        if totals.remaining_sell.lte(zero)? {
            break;
        }

        if let Some(leg) = take_leg(candidate, totals.remaining_sell)? {
            apply_leg(&leg, &mut totals)?;
            legs.push(leg);
        }
    }

    Ok(SimulatedSellResult {
        legs,
        total_buy_amount: totals.total_buy_amount,
        total_sell_amount: totals.total_sell_amount,
    })
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::test_helpers::candidates::make_simulation_candidate;
    use rain_math_float::Float;

    #[test]
    fn test_simulate_exact_in_sell_empty_candidates() {
        let candidates: Vec<TakeOrderCandidate> = vec![];
        let sell_budget = Float::parse("1.5".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert!(result.legs.is_empty());
        assert!(result.total_buy_amount.eq(Float::zero().unwrap()).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_simulate_exact_in_sell_single_candidate_full_fill() {
        let f1_5 = Float::parse("1.5".to_string()).unwrap();
        let f2_25 = Float::parse("2.25".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_25, f1_5);
        let candidates = vec![candidate];
        let sell_budget = Float::parse("3.375".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result.total_buy_amount.eq(f2_25).unwrap());
        assert!(result
            .total_sell_amount
            .eq(Float::parse("3.375".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_simulate_exact_in_sell_single_candidate_partial_fill() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f4_5 = Float::parse("4.5".to_string()).unwrap();
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let candidate = make_simulation_candidate(f4_5, f1_25);
        let candidates = vec![candidate];
        let sell_budget = f2_5;

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result
            .total_buy_amount
            .eq(Float::parse("2".to_string()).unwrap())
            .unwrap());
        assert!(result.total_sell_amount.eq(f2_5).unwrap());
    }

    #[test]
    fn test_simulate_exact_in_sell_multiple_candidates_sorted_by_price() {
        let f1_5 = Float::parse("1.5".to_string()).unwrap();
        let f2_75 = Float::parse("2.75".to_string()).unwrap();
        let f3_25 = Float::parse("3.25".to_string()).unwrap();
        let expensive = make_simulation_candidate(f2_75, f3_25);
        let cheap = make_simulation_candidate(f2_75, f1_5);
        let candidates = vec![expensive, cheap];
        let sell_budget = Float::parse("4.125".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(
            result.legs[0].candidate.ratio.eq(f1_5).unwrap(),
            "Should use cheapest candidate first"
        );
        assert!(result.total_buy_amount.eq(f2_75).unwrap());
        assert!(result
            .total_sell_amount
            .eq(Float::parse("4.125".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_simulate_exact_in_sell_multiple_candidates_uses_multiple() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let cheap = make_simulation_candidate(f1_25, f1_25);
        let expensive = make_simulation_candidate(f2_5, f2_5);
        let candidates = vec![expensive, cheap];
        let sell_budget = Float::parse("8".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 2, "Should use both candidates");
        assert!(
            result.legs[0].candidate.ratio.eq(f1_25).unwrap(),
            "First leg should be cheapest"
        );
        assert!(
            result.legs[1].candidate.ratio.eq(f2_5).unwrap(),
            "Second leg should be more expensive"
        );
    }

    #[test]
    fn test_simulate_exact_in_sell_zero_budget() {
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let f1_75 = Float::parse("1.75".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_5, f1_75);
        let candidates = vec![candidate];
        let sell_budget = Float::zero().unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert!(result.legs.is_empty());
        assert!(result.total_buy_amount.eq(Float::zero().unwrap()).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_simulate_exact_in_sell_with_different_ratio() {
        let f0_5 = Float::parse("0.5".to_string()).unwrap();
        let f4_5 = Float::parse("4.5".to_string()).unwrap();
        let f2_25 = Float::parse("2.25".to_string()).unwrap();
        let candidate = make_simulation_candidate(f4_5, f0_5);
        let candidates = vec![candidate];
        let sell_budget = f2_25;

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result.total_buy_amount.eq(f4_5).unwrap());
        assert!(result.total_sell_amount.eq(f2_25).unwrap());
    }

    #[test]
    fn test_simulate_multi_leg_partial_fill_second_leg() {
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

        assert_eq!(result.legs.len(), 2, "Should use exactly 2 legs");

        assert!(
            result.legs[0].candidate.ratio.eq(ratio_1).unwrap(),
            "First leg should be cheapest (ratio=1)"
        );
        assert!(
            result.legs[1].candidate.ratio.eq(ratio_2).unwrap(),
            "Second leg should be mid-price (ratio=2)"
        );

        let expected_leg1_buy = Float::parse("100".to_string()).unwrap();
        let expected_leg1_sell = Float::parse("100".to_string()).unwrap();
        assert!(
            result.legs[0].buy_amount.eq(expected_leg1_buy).unwrap(),
            "Leg 1 buy_amount should be 100 (full fill)"
        );
        assert!(
            result.legs[0].sell_amount.eq(expected_leg1_sell).unwrap(),
            "Leg 1 sell_amount should be 100"
        );

        let expected_leg2_buy = Float::parse("25".to_string()).unwrap();
        let expected_leg2_sell = Float::parse("50".to_string()).unwrap();
        assert!(
            result.legs[1].buy_amount.eq(expected_leg2_buy).unwrap(),
            "Leg 2 buy_amount should be 25 (partial: 50 / 2)"
        );
        assert!(
            result.legs[1].sell_amount.eq(expected_leg2_sell).unwrap(),
            "Leg 2 sell_amount should be 50 (remaining budget)"
        );

        let expected_total_buy = Float::parse("125".to_string()).unwrap();
        assert!(
            result.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should be 125 (100 + 25)"
        );
        assert!(
            result.total_sell_amount.eq(sell_budget).unwrap(),
            "total_sell_amount should equal sell_budget (150)"
        );
    }

    #[test]
    fn test_take_leg_full_fill() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let remaining_sell = Float::parse("300".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_sell).unwrap().unwrap();

        assert!(
            leg.buy_amount.eq(max_output).unwrap(),
            "Full fill: buy_amount should equal max_output"
        );
        let expected_sell = Float::parse("200".to_string()).unwrap();
        assert!(
            leg.sell_amount.eq(expected_sell).unwrap(),
            "Full fill: sell_amount should be max_output * ratio"
        );
    }

    #[test]
    fn test_take_leg_partial_fill() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let remaining_sell = Float::parse("50".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_sell).unwrap().unwrap();

        let expected_buy = Float::parse("25".to_string()).unwrap();
        assert!(
            leg.buy_amount.eq(expected_buy).unwrap(),
            "Partial fill: buy_amount should be remaining_sell / ratio"
        );
        assert!(
            leg.sell_amount.eq(remaining_sell).unwrap(),
            "Partial fill: sell_amount should equal remaining_sell"
        );
    }

    #[test]
    fn test_take_leg_zero_price_full_fill() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let zero_ratio = Float::zero().unwrap();
        let candidate = make_simulation_candidate(max_output, zero_ratio);
        let remaining_sell = Float::parse("50".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_sell).unwrap().unwrap();

        assert!(
            leg.buy_amount.eq(max_output).unwrap(),
            "Zero-price: full fill with buy_amount = max_output (free tokens)"
        );
        assert!(
            leg.sell_amount.eq(Float::zero().unwrap()).unwrap(),
            "Zero-price: sell_amount should be 0"
        );
    }

    #[test]
    fn test_take_leg_zero_remaining_sell() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let remaining_sell = Float::zero().unwrap();

        let result = take_leg(candidate, remaining_sell).unwrap();

        assert!(result.is_none(), "Zero remaining sell should return None");
    }

    #[test]
    fn test_sort_candidates_by_price() {
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let ratio_3 = Float::parse("3".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let mut candidates = vec![
            make_simulation_candidate(max_output, ratio_3),
            make_simulation_candidate(max_output, ratio_1),
            make_simulation_candidate(max_output, ratio_2),
        ];

        sort_candidates_by_price(&mut candidates).unwrap();

        assert!(
            candidates[0].ratio.eq(ratio_1).unwrap(),
            "First candidate should have lowest ratio"
        );
        assert!(
            candidates[1].ratio.eq(ratio_2).unwrap(),
            "Second candidate should have middle ratio"
        );
        assert!(
            candidates[2].ratio.eq(ratio_3).unwrap(),
            "Third candidate should have highest ratio"
        );
    }

    #[test]
    fn test_simulate_zero_price_candidate_included() {
        let zero_ratio = Float::zero().unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let zero_price_candidate = make_simulation_candidate(max_output, zero_ratio);
        let normal_candidate = make_simulation_candidate(max_output, ratio_2);

        let candidates = vec![normal_candidate, zero_price_candidate];
        let sell_budget = Float::parse("200".to_string()).unwrap();

        let result = simulate_sell_over_candidates(candidates, sell_budget).unwrap();

        assert_eq!(result.legs.len(), 2, "Both candidates should be used");
        assert!(
            result.legs[0].candidate.ratio.eq(zero_ratio).unwrap(),
            "First leg should be zero-price (sorted first)"
        );
        assert!(
            result.legs[1].candidate.ratio.eq(ratio_2).unwrap(),
            "Second leg should be normal candidate with ratio=2"
        );
        let expected_total_buy = Float::parse("200".to_string()).unwrap();
        assert!(
            result.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should equal 200 (100 + 100)"
        );
        assert!(
            result.total_sell_amount.eq(sell_budget).unwrap(),
            "total_sell_amount should equal sell_budget (0 for zero-price + 200 for normal)"
        );
    }

    #[test]
    fn test_apply_leg_updates_totals() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let buy_amount = Float::parse("50".to_string()).unwrap();
        let sell_amount = Float::parse("100".to_string()).unwrap();

        let leg = SelectedTakeOrderLeg {
            candidate,
            buy_amount,
            sell_amount,
        };

        let mut totals = SimulationTotals {
            remaining_sell: Float::parse("200".to_string()).unwrap(),
            total_buy_amount: Float::parse("25".to_string()).unwrap(),
            total_sell_amount: Float::parse("50".to_string()).unwrap(),
        };

        apply_leg(&leg, &mut totals).unwrap();

        let expected_remaining = Float::parse("100".to_string()).unwrap();
        let expected_total_buy = Float::parse("75".to_string()).unwrap();
        let expected_total_sell = Float::parse("150".to_string()).unwrap();

        assert!(
            totals.remaining_sell.eq(expected_remaining).unwrap(),
            "remaining_sell should be reduced by sell_amount"
        );
        assert!(
            totals.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should be increased by buy_amount"
        );
        assert!(
            totals.total_sell_amount.eq(expected_total_sell).unwrap(),
            "total_sell_amount should be increased by sell_amount"
        );
    }
}
