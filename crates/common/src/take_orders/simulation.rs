use super::candidates::TakeOrderCandidate;
use super::price::cmp_float;
use crate::raindex_client::RaindexError;
use rain_math_float::Float;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Debug)]
pub struct SelectedTakeOrderLeg {
    pub candidate: TakeOrderCandidate,
    pub buy_amount: Float,
    pub sell_amount: Float,
}

#[derive(Clone, Debug)]
pub struct SimulatedBuyResult {
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
    remaining_buy: Float,
) -> Result<Option<SelectedTakeOrderLeg>, RaindexError> {
    let zero = Float::zero()?;
    let price = candidate.ratio;
    let max_buy = candidate.max_output;

    let buy_amount = if max_buy.lte(remaining_buy)? {
        max_buy
    } else {
        remaining_buy
    };

    if buy_amount.lte(zero)? {
        return Ok(None);
    }

    let sell_amount = buy_amount.mul(price)?;

    Ok(Some(SelectedTakeOrderLeg {
        candidate,
        buy_amount,
        sell_amount,
    }))
}

struct SimulationTotals {
    remaining_buy: Float,
    total_buy_amount: Float,
    total_sell_amount: Float,
}

fn apply_leg(
    leg: &SelectedTakeOrderLeg,
    totals: &mut SimulationTotals,
) -> Result<(), RaindexError> {
    totals.remaining_buy = totals.remaining_buy.sub(leg.buy_amount)?;
    totals.total_buy_amount = totals.total_buy_amount.add(leg.buy_amount)?;
    totals.total_sell_amount = totals.total_sell_amount.add(leg.sell_amount)?;
    Ok(())
}

fn filter_candidates_by_price_cap(
    candidates: Vec<TakeOrderCandidate>,
    price_cap: Float,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    Ok(candidates
        .into_iter()
        .map(|candidate| {
            candidate
                .ratio
                .lte(price_cap)
                .map(|is_below_cap| if is_below_cap { Some(candidate) } else { None })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

pub fn simulate_buy_over_candidates(
    candidates: Vec<TakeOrderCandidate>,
    buy_target: Float,
    price_cap: Float,
) -> Result<SimulatedBuyResult, RaindexError> {
    let mut filtered = filter_candidates_by_price_cap(candidates, price_cap)?;
    sort_candidates_by_price(&mut filtered)?;

    let zero = Float::zero()?;
    let mut totals = SimulationTotals {
        remaining_buy: buy_target,
        total_buy_amount: zero,
        total_sell_amount: zero,
    };
    let mut legs: Vec<SelectedTakeOrderLeg> = Vec::new();

    for candidate in filtered {
        if totals.remaining_buy.lte(zero)? {
            break;
        }

        if let Some(leg) = take_leg(candidate, totals.remaining_buy)? {
            apply_leg(&leg, &mut totals)?;
            legs.push(leg);
        }
    }

    Ok(SimulatedBuyResult {
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

    fn high_price_cap() -> Float {
        Float::parse("1000000".to_string()).unwrap()
    }

    #[test]
    fn test_simulate_buy_empty_candidates() {
        let candidates: Vec<TakeOrderCandidate> = vec![];
        let buy_target = Float::parse("1.5".to_string()).unwrap();

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert!(result.legs.is_empty());
        assert!(result.total_buy_amount.eq(Float::zero().unwrap()).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_simulate_buy_single_candidate_full_fill() {
        let f1_5 = Float::parse("1.5".to_string()).unwrap();
        let f2_25 = Float::parse("2.25".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_25, f1_5);
        let candidates = vec![candidate];
        let buy_target = f2_25;

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result.total_buy_amount.eq(f2_25).unwrap());
        let expected_sell = Float::parse("3.375".to_string()).unwrap();
        assert!(result.total_sell_amount.eq(expected_sell).unwrap());
    }

    #[test]
    fn test_simulate_buy_single_candidate_partial_fill() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f4_5 = Float::parse("4.5".to_string()).unwrap();
        let f2_0 = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(f4_5, f1_25);
        let candidates = vec![candidate];
        let buy_target = f2_0;

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result.total_buy_amount.eq(f2_0).unwrap());
        let expected_sell = Float::parse("2.5".to_string()).unwrap();
        assert!(result.total_sell_amount.eq(expected_sell).unwrap());
    }

    #[test]
    fn test_simulate_buy_multiple_candidates_sorted_by_price() {
        let f1_5 = Float::parse("1.5".to_string()).unwrap();
        let f2_75 = Float::parse("2.75".to_string()).unwrap();
        let f3_25 = Float::parse("3.25".to_string()).unwrap();
        let expensive = make_simulation_candidate(f2_75, f3_25);
        let cheap = make_simulation_candidate(f2_75, f1_5);
        let candidates = vec![expensive, cheap];
        let buy_target = f2_75;

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(
            result.legs[0].candidate.ratio.eq(f1_5).unwrap(),
            "Should use cheapest candidate first"
        );
        assert!(result.total_buy_amount.eq(f2_75).unwrap());
        let expected_sell = Float::parse("4.125".to_string()).unwrap();
        assert!(result.total_sell_amount.eq(expected_sell).unwrap());
    }

    #[test]
    fn test_simulate_buy_multiple_candidates_uses_multiple() {
        let f1_25 = Float::parse("1.25".to_string()).unwrap();
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let f3_75 = Float::parse("3.75".to_string()).unwrap();
        let cheap = make_simulation_candidate(f1_25, f1_25);
        let expensive = make_simulation_candidate(f2_5, f2_5);
        let candidates = vec![expensive, cheap];
        let buy_target = f3_75;

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert_eq!(result.legs.len(), 2, "Should use both candidates");
        assert!(
            result.legs[0].candidate.ratio.eq(f1_25).unwrap(),
            "First leg should be cheapest"
        );
        assert!(
            result.legs[1].candidate.ratio.eq(f2_5).unwrap(),
            "Second leg should be more expensive"
        );
        assert!(result.total_buy_amount.eq(f3_75).unwrap());
    }

    #[test]
    fn test_simulate_buy_zero_target() {
        let f2_5 = Float::parse("2.5".to_string()).unwrap();
        let f1_75 = Float::parse("1.75".to_string()).unwrap();
        let candidate = make_simulation_candidate(f2_5, f1_75);
        let candidates = vec![candidate];
        let buy_target = Float::zero().unwrap();

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert!(result.legs.is_empty());
        assert!(result.total_buy_amount.eq(Float::zero().unwrap()).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_simulate_buy_with_different_ratio() {
        let f0_5 = Float::parse("0.5".to_string()).unwrap();
        let f4_5 = Float::parse("4.5".to_string()).unwrap();
        let candidate = make_simulation_candidate(f4_5, f0_5);
        let candidates = vec![candidate];
        let buy_target = f4_5;

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

        assert_eq!(result.legs.len(), 1);
        assert!(result.total_buy_amount.eq(f4_5).unwrap());
        let expected_sell = Float::parse("2.25".to_string()).unwrap();
        assert!(result.total_sell_amount.eq(expected_sell).unwrap());
    }

    #[test]
    fn test_simulate_buy_multi_leg_partial_fill_second_leg() {
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let ratio_3 = Float::parse("3".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let candidate_cheap = make_simulation_candidate(max_output, ratio_1);
        let candidate_mid = make_simulation_candidate(max_output, ratio_2);
        let candidate_expensive = make_simulation_candidate(max_output, ratio_3);

        let candidates = vec![candidate_expensive, candidate_mid, candidate_cheap];
        let buy_target = Float::parse("125".to_string()).unwrap();

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

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
            "Leg 2 buy_amount should be 25 (partial)"
        );
        assert!(
            result.legs[1].sell_amount.eq(expected_leg2_sell).unwrap(),
            "Leg 2 sell_amount should be 50 (25 * 2)"
        );

        let expected_total_buy = Float::parse("125".to_string()).unwrap();
        let expected_total_sell = Float::parse("150".to_string()).unwrap();
        assert!(
            result.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should be 125 (100 + 25)"
        );
        assert!(
            result.total_sell_amount.eq(expected_total_sell).unwrap(),
            "total_sell_amount should be 150 (100 + 50)"
        );
    }

    #[test]
    fn test_take_leg_full_fill() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let remaining_buy = Float::parse("200".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_buy).unwrap().unwrap();

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
        let remaining_buy = Float::parse("50".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_buy).unwrap().unwrap();

        assert!(
            leg.buy_amount.eq(remaining_buy).unwrap(),
            "Partial fill: buy_amount should equal remaining_buy"
        );
        let expected_sell = Float::parse("100".to_string()).unwrap();
        assert!(
            leg.sell_amount.eq(expected_sell).unwrap(),
            "Partial fill: sell_amount should be buy_amount * ratio"
        );
    }

    #[test]
    fn test_take_leg_zero_price_full_fill() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let zero_ratio = Float::zero().unwrap();
        let candidate = make_simulation_candidate(max_output, zero_ratio);
        let remaining_buy = Float::parse("200".to_string()).unwrap();

        let leg = take_leg(candidate, remaining_buy).unwrap().unwrap();

        assert!(
            leg.buy_amount.eq(max_output).unwrap(),
            "Zero-price: buy_amount should equal max_output (capped by capacity)"
        );
        assert!(
            leg.sell_amount.eq(Float::zero().unwrap()).unwrap(),
            "Zero-price: sell_amount should be 0"
        );
    }

    #[test]
    fn test_take_leg_zero_remaining_buy() {
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_simulation_candidate(max_output, ratio);
        let remaining_buy = Float::zero().unwrap();

        let result = take_leg(candidate, remaining_buy).unwrap();

        assert!(result.is_none(), "Zero remaining buy should return None");
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
    fn test_simulate_buy_zero_price_candidate_included() {
        let zero_ratio = Float::zero().unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let zero_price_candidate = make_simulation_candidate(max_output, zero_ratio);
        let normal_candidate = make_simulation_candidate(max_output, ratio_2);

        let candidates = vec![normal_candidate, zero_price_candidate];
        let buy_target = Float::parse("200".to_string()).unwrap();

        let result =
            simulate_buy_over_candidates(candidates, buy_target, high_price_cap()).unwrap();

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
        let expected_total_sell = Float::parse("200".to_string()).unwrap();
        assert!(
            result.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should equal 200 (100 + 100)"
        );
        assert!(
            result.total_sell_amount.eq(expected_total_sell).unwrap(),
            "total_sell_amount should be 200 (0 for zero-price + 200 for normal)"
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
            remaining_buy: Float::parse("200".to_string()).unwrap(),
            total_buy_amount: Float::parse("25".to_string()).unwrap(),
            total_sell_amount: Float::parse("50".to_string()).unwrap(),
        };

        apply_leg(&leg, &mut totals).unwrap();

        let expected_remaining = Float::parse("150".to_string()).unwrap();
        let expected_total_buy = Float::parse("75".to_string()).unwrap();
        let expected_total_sell = Float::parse("150".to_string()).unwrap();

        assert!(
            totals.remaining_buy.eq(expected_remaining).unwrap(),
            "remaining_buy should be reduced by buy_amount"
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

    #[test]
    fn test_filter_candidates_by_price_cap() {
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let ratio_3 = Float::parse("3".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let candidate_1 = make_simulation_candidate(max_output, ratio_1);
        let candidate_2 = make_simulation_candidate(max_output, ratio_2);
        let candidate_3 = make_simulation_candidate(max_output, ratio_3);

        let candidates = vec![candidate_1, candidate_2, candidate_3];
        let price_cap = Float::parse("2".to_string()).unwrap();

        let filtered = filter_candidates_by_price_cap(candidates, price_cap).unwrap();

        assert_eq!(
            filtered.len(),
            2,
            "Only candidates with ratio <= 2 should pass"
        );
        assert!(filtered[0].ratio.eq(ratio_1).unwrap());
        assert!(filtered[1].ratio.eq(ratio_2).unwrap());
    }

    #[test]
    fn test_simulate_buy_with_price_cap_filters_expensive() {
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let ratio_2 = Float::parse("2".to_string()).unwrap();
        let ratio_3 = Float::parse("3".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let candidate_1 = make_simulation_candidate(max_output, ratio_1);
        let candidate_2 = make_simulation_candidate(max_output, ratio_2);
        let candidate_3 = make_simulation_candidate(max_output, ratio_3);

        let candidates = vec![candidate_3, candidate_2, candidate_1];
        let buy_target = Float::parse("300".to_string()).unwrap();
        let price_cap = Float::parse("2".to_string()).unwrap();

        let result = simulate_buy_over_candidates(candidates, buy_target, price_cap).unwrap();

        assert_eq!(
            result.legs.len(),
            2,
            "Only 2 candidates should be used (ratio <= 2)"
        );
        assert!(
            result.legs[0].candidate.ratio.eq(ratio_1).unwrap(),
            "First leg should be cheapest"
        );
        assert!(
            result.legs[1].candidate.ratio.eq(ratio_2).unwrap(),
            "Second leg should be at price cap"
        );
        let expected_total_buy = Float::parse("200".to_string()).unwrap();
        assert!(
            result.total_buy_amount.eq(expected_total_buy).unwrap(),
            "total_buy_amount should be 200 (100 + 100), not 300"
        );
    }

    #[test]
    fn test_simulate_buy_price_cap_zero_only_free_orders() {
        let zero_ratio = Float::zero().unwrap();
        let ratio_1 = Float::parse("1".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let free_candidate = make_simulation_candidate(max_output, zero_ratio);
        let paid_candidate = make_simulation_candidate(max_output, ratio_1);

        let candidates = vec![paid_candidate, free_candidate];
        let buy_target = Float::parse("200".to_string()).unwrap();
        let price_cap = Float::zero().unwrap();

        let result = simulate_buy_over_candidates(candidates, buy_target, price_cap).unwrap();

        assert_eq!(
            result.legs.len(),
            1,
            "Only zero-price candidate should be used"
        );
        assert!(result.legs[0].candidate.ratio.eq(zero_ratio).unwrap());
        let expected_total_buy = Float::parse("100".to_string()).unwrap();
        assert!(result.total_buy_amount.eq(expected_total_buy).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_simulate_buy_all_candidates_above_price_cap() {
        let ratio_5 = Float::parse("5".to_string()).unwrap();
        let ratio_10 = Float::parse("10".to_string()).unwrap();
        let max_output = Float::parse("100".to_string()).unwrap();

        let candidate_1 = make_simulation_candidate(max_output, ratio_5);
        let candidate_2 = make_simulation_candidate(max_output, ratio_10);

        let candidates = vec![candidate_1, candidate_2];
        let buy_target = Float::parse("100".to_string()).unwrap();
        let price_cap = Float::parse("2".to_string()).unwrap();

        let result = simulate_buy_over_candidates(candidates, buy_target, price_cap).unwrap();

        assert!(
            result.legs.is_empty(),
            "No candidates should pass the price cap filter"
        );
        assert!(result.total_buy_amount.eq(Float::zero().unwrap()).unwrap());
        assert!(result.total_sell_amount.eq(Float::zero().unwrap()).unwrap());
    }
}
