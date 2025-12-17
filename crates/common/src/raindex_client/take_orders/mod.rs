mod request;
mod result;
mod selection;

#[cfg(all(test, not(target_family = "wasm")))]
mod e2e_tests;

pub use result::TakeOrdersCalldataResult;

use super::orders::{GetOrdersFilters, GetOrdersTokenFilter, RaindexOrder};
use super::{ChainIds, RaindexClient, RaindexError};
use crate::rpc_client::RpcClient;
use crate::take_orders::{build_take_orders_config_from_buy_simulation, MinReceiveMode};
use alloy::primitives::Address;
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
    /// - `buyAmount`: human-readable decimal string in buy-token units; parsed with `Float::parse`.
    /// - `priceCap`: human-readable decimal string for max sell per 1 buy; parsed with `Float::parse`.
    /// - `minReceiveMode`:
    ///   - `Partial` → `minimumInput = 0` (may underfill).
    ///   - `Exact`   → `minimumInput = buyAmount` (error if liquidity < buyAmount).
    ///
    /// Returns calldata plus pricing info:
    /// - `calldata`: ABI-encoded bytes for `takeOrders3`.
    /// - `effectivePrice`: expected blended sell per 1 buy from the simulation.
    /// - `prices`: per-leg ratios, best→worst.
    /// - `expectedSell`: simulated sell at current quotes.
    /// - `maxSellCap`: `buyAmount * priceCap` (worst-case on-chain spend cap).
    ///
    /// ## Example (JS)
    /// ```javascript
    /// const res = await client.getTakeOrdersCalldata(
    ///   137,
    ///   "0xSELL...", // sellToken
    ///   "0xBUY...",  // buyToken
    ///   "10",        // buyAmount (decimal string)
    ///   "1.2",       // priceCap (decimal string, sell per 1 buy)
    ///   "partial"    // or "exact"
    /// );
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
            js_name = "buyAmount",
            param_description = "Human-readable amount in buyToken units (e.g., \"10.5\")",
            unchecked_param_type = "string"
        )]
        buy_amount: String,
        #[wasm_export(
            js_name = "priceCap",
            param_description = "Human-readable max price (sell per 1 buy), e.g., \"1.2\"",
            unchecked_param_type = "string"
        )]
        price_cap: String,
        #[wasm_export(
            js_name = "minReceiveMode",
            param_description = "Minimum receive policy: partial or exact"
        )]
        min_receive_mode: MinReceiveMode,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let req = request::parse_request(
            &sell_token,
            &buy_token,
            &buy_amount,
            &price_cap,
            min_receive_mode,
        )?;

        let orders = self
            .fetch_orders_for_pair(chain_id, req.sell_token, req.buy_token)
            .await?;

        let rpc_urls = self.get_rpc_urls_for_chain(chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls)?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates = selection::build_candidates_for_chain(
            &orders,
            req.sell_token,
            req.buy_token,
            Some(block_number),
        )
        .await?;

        let (best_orderbook, best_sim) =
            selection::select_best_orderbook_simulation(candidates, req.buy_amount, req.price_cap)?;

        let built = build_take_orders_config_from_buy_simulation(
            best_sim,
            req.buy_amount,
            req.price_cap,
            req.min_receive_mode,
        )?
        .ok_or(RaindexError::NoLiquidity)?;

        result::build_calldata_result(best_orderbook, built, req.buy_amount, req.price_cap)
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
}
