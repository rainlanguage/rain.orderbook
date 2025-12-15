use crate::raindex_client::orders::RaindexOrder;
use crate::raindex_client::RaindexError;
use crate::take_orders::{
    build_take_order_candidates_for_pair, cmp_float, simulate_sell_over_candidates,
    SimulatedSellResult, TakeOrderCandidate,
};
use alloy::primitives::Address;
use rain_math_float::Float;
use std::collections::HashMap;

pub(crate) async fn build_candidates_for_chain(
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

pub(crate) fn worst_price(sim: &SimulatedSellResult) -> Result<Option<Float>, RaindexError> {
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

pub(crate) fn select_best_orderbook_simulation(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::candidates::make_candidate;

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
}
