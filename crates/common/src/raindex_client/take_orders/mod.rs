mod request;
mod result;
mod selection;

#[cfg(all(test, not(target_family = "wasm")))]
mod e2e_tests;

pub use request::TakeOrdersRequest;
pub use result::TakeOrdersCalldataResult;

use super::orders::{GetOrdersFilters, GetOrdersTokenFilter, RaindexOrder};
use super::{ChainIds, RaindexClient, RaindexError};
use crate::erc20::ERC20;
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_orders_config_from_buy_simulation, check_taker_balance_and_allowance,
    find_failing_order_index, simulate_take_orders, TakeOrderCandidate,
};
use alloy::primitives::Address;
use rain_orderbook_bindings::provider::mk_read_provider;
use std::ops::Mul;
use wasm_bindgen_utils::prelude::*;
use wasm_bindgen_utils::wasm_export;

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

#[wasm_export]
impl RaindexClient {
    /// Generates calldata for `IOrderBookV5.takeOrders3` using a buy-target + price-cap policy.
    ///
    /// This function:
    /// 1. Fetches orders matching the token pair
    /// 2. Checks taker has sufficient balance and allowance for worst-case spend
    /// 3. Builds the optimal order config via simulation
    /// 4. Runs preflight `eth_call` to validate the transaction
    /// 5. If preflight fails, identifies and removes failing orders, then retries
    /// 6. Returns the final validated calldata
    ///
    /// ## Parameters
    /// - `request`: A `TakeOrdersRequest` object containing:
    ///   - `taker`: Address of the account executing the trade
    ///   - `chainId`: Chain ID of the target network
    ///   - `sellToken`: Token address the taker will GIVE
    ///   - `buyToken`: Token address the taker will RECEIVE
    ///   - `buyAmount`: Human-readable decimal string (e.g., "10.5")
    ///   - `priceCap`: Human-readable max price (sell per 1 buy), e.g., "1.2"
    ///   - `minReceiveMode`: `partial` (may underfill) or `exact` (error if insufficient)
    ///
    /// ## Returns
    /// - `calldata`: ABI-encoded bytes for `takeOrders3`
    /// - `effectivePrice`: expected blended sell per 1 buy
    /// - `prices`: per-leg ratios, best→worst
    /// - `expectedSell`: simulated sell at current quotes
    /// - `maxSellCap`: `buyAmount * priceCap` (worst-case spend cap)
    ///
    /// ## Example (JS)
    /// ```javascript
    /// const res = await client.getTakeOrdersCalldata({
    ///   taker: "0xTAKER...",
    ///   chainId: 137,
    ///   sellToken: "0xSELL...",
    ///   buyToken: "0xBUY...",
    ///   buyAmount: "10",
    ///   priceCap: "1.2",
    ///   minReceiveMode: "partial"
    /// });
    /// if (res.error) {
    ///   console.error(res.error.readableMsg);
    /// } else {
    ///   const { calldata, effectivePrice, expectedSell, maxSellCap, prices, orderbook } = res.value;
    /// }
    /// ```
    #[wasm_export(
        js_name = "getTakeOrdersCalldata",
        return_description = "Encoded takeOrders3 calldata and price information",
        unchecked_return_type = "TakeOrdersCalldataResult"
    )]
    pub async fn get_take_orders_calldata(
        &self,
        #[wasm_export(
            js_name = "request",
            param_description = "Request parameters for take orders"
        )]
        request: TakeOrdersRequest,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let req = request::parse_request_from_struct(&request)?;

        let orders = self
            .fetch_orders_for_pair(request.chain_id, req.sell_token, req.buy_token)
            .await?;

        let rpc_urls = self.get_rpc_urls_for_chain(request.chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls.clone())?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let max_sell = req.buy_amount.mul(req.price_cap)?;
        let erc20 = ERC20::new(rpc_urls.clone(), req.sell_token);

        let mut candidates = selection::build_candidates_for_chain(
            &orders,
            req.sell_token,
            req.buy_token,
            Some(block_number),
        )
        .await?;

        let provider =
            mk_read_provider(&rpc_urls).map_err(|e| RaindexError::PreflightError(e.to_string()))?;

        let mut balance_checked_for_orderbook: Option<Address> = None;

        loop {
            let (best_orderbook, best_sim) = selection::select_best_orderbook_simulation(
                candidates.clone(),
                req.buy_amount,
                req.price_cap,
            )?;

            if balance_checked_for_orderbook != Some(best_orderbook) {
                check_taker_balance_and_allowance(&erc20, req.taker, best_orderbook, max_sell)
                    .await
                    .map_err(|e| RaindexError::PreflightError(e.to_string()))?;
                balance_checked_for_orderbook = Some(best_orderbook);
            }

            let built = build_take_orders_config_from_buy_simulation(
                best_sim.clone(),
                req.buy_amount,
                req.price_cap,
                req.min_receive_mode,
            )?
            .ok_or(RaindexError::NoLiquidity)?;

            let simulation_result = simulate_take_orders(
                &provider,
                best_orderbook,
                req.taker,
                &built.config,
                block_number,
            )
            .await;

            if simulation_result.is_ok() {
                return result::build_calldata_result(
                    best_orderbook,
                    built,
                    req.buy_amount,
                    req.price_cap,
                );
            }

            let failing_index = find_failing_order_index(
                &provider,
                best_orderbook,
                req.taker,
                &built.config,
                block_number,
            )
            .await;

            match failing_index {
                Some(idx) => {
                    let failing_order = &built.config.orders[idx];
                    candidates = remove_candidate_by_order(&candidates, failing_order);

                    if candidates.is_empty() {
                        return Err(RaindexError::NoLiquidity);
                    }
                }
                None => {
                    return Err(RaindexError::PreflightError(
                        "Simulation failed but could not identify failing order".to_string(),
                    ));
                }
            }
        }
    }
}

fn remove_candidate_by_order(
    candidates: &[TakeOrderCandidate],
    failing_order: &rain_orderbook_bindings::IOrderBookV5::TakeOrderConfigV4,
) -> Vec<TakeOrderCandidate> {
    let input_idx: u32 = failing_order.inputIOIndex.try_into().unwrap_or(u32::MAX);
    let output_idx: u32 = failing_order.outputIOIndex.try_into().unwrap_or(u32::MAX);

    candidates
        .iter()
        .filter(|c| {
            c.order != failing_order.order
                || c.input_io_index != input_idx
                || c.output_io_index != output_idx
        })
        .cloned()
        .collect()
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
}
