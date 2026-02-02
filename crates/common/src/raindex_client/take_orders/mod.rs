mod request;
mod result;
mod selection;

#[cfg(all(test, not(target_family = "wasm")))]
mod e2e_tests;

pub use result::TakeOrdersCalldataResult;

use super::orders::{GetOrdersFilters, GetOrdersTokenFilter, RaindexOrder};
use super::{ChainIds, RaindexClient, RaindexError};
use crate::rpc_client::RpcClient;
use crate::take_orders::{build_take_orders_config_from_simulation, TakeOrdersMode};
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
            orderbook_addresses: None,
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
    /// Generates calldata for `IOrderBookV6.takeOrders4` using a mode + price-cap policy.
    ///
    /// - `mode`: One of `buyExact`, `buyUpTo`, `spendExact`, or `spendUpTo`
    /// - `amount`: Target amount (output tokens for buy modes, input tokens for spend modes)
    /// - `priceCap`: human-readable decimal string for max sell per 1 buy; parsed with `Float::parse`.
    ///
    /// Returns calldata plus pricing info:
    /// - `calldata`: ABI-encoded bytes for `takeOrders4`.
    /// - `effectivePrice`: expected blended sell per 1 buy from the simulation.
    /// - `prices`: per-leg ratios, best→worst.
    /// - `expectedSell`: simulated sell at current quotes.
    /// - `maxSellCap`: `amount * priceCap` for buy modes, `amount` for spend modes (worst-case on-chain spend cap).
    ///
    /// ## Example (JS)
    /// ```javascript
    /// const res = await client.getTakeOrdersCalldata(
    ///   137,
    ///   "0xSELL...", // sellToken
    ///   "0xBUY...",  // buyToken
    ///   "buyUpTo",   // mode
    ///   "10",        // amount
    ///   "1.2",       // priceCap (decimal string, sell per 1 buy)
    /// );
    /// if (res.error) {
    ///   console.error(res.error.readableMsg);
    /// } else {
    ///   const { calldata, effectivePrice, expectedSell, maxSellCap, prices, orderbook } = res.value;
    /// }
    /// ```
    #[wasm_export(
        js_name = "getTakeOrdersCalldata",
        return_description = "Encoded takeOrders4 calldata and price information",
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
            js_name = "mode",
            param_description = "Take orders mode: buyExact, buyUpTo, spendExact, or spendUpTo"
        )]
        mode: TakeOrdersMode,
        #[wasm_export(
            js_name = "amount",
            param_description = "Target amount (output for buy modes, input for spend modes)",
            unchecked_param_type = "string"
        )]
        amount: String,
        #[wasm_export(
            js_name = "priceCap",
            param_description = "Human-readable max price (sell per 1 buy), e.g., \"1.2\"",
            unchecked_param_type = "string"
        )]
        price_cap: String,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let req = request::parse_request(&sell_token, &buy_token, mode, &amount, &price_cap)?;

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
            selection::select_best_orderbook_simulation(candidates, req.mode, req.price_cap)?;

        let built = build_take_orders_config_from_simulation(best_sim, req.mode, req.price_cap)?
            .ok_or(RaindexError::NoLiquidity)?;

        result::build_calldata_result(best_orderbook, built, req.mode, req.price_cap)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::take_orders::TakeOrdersMode;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::{from_js_value, to_js_value};

        #[wasm_bindgen_test]
        fn test_take_orders_mode_serialization() {
            let buy_up_to = TakeOrdersMode::BuyUpTo;
            let buy_exact = TakeOrdersMode::BuyExact;
            let spend_up_to = TakeOrdersMode::SpendUpTo;
            let spend_exact = TakeOrdersMode::SpendExact;

            let buy_up_to_js = to_js_value(&buy_up_to).unwrap();
            let buy_exact_js = to_js_value(&buy_exact).unwrap();
            let spend_up_to_js = to_js_value(&spend_up_to).unwrap();
            let spend_exact_js = to_js_value(&spend_exact).unwrap();

            let buy_up_to_back: TakeOrdersMode = from_js_value(buy_up_to_js).unwrap();
            let buy_exact_back: TakeOrdersMode = from_js_value(buy_exact_js).unwrap();
            let spend_up_to_back: TakeOrdersMode = from_js_value(spend_up_to_js).unwrap();
            let spend_exact_back: TakeOrdersMode = from_js_value(spend_exact_js).unwrap();

            assert_eq!(buy_up_to_back, TakeOrdersMode::BuyUpTo);
            assert_eq!(buy_exact_back, TakeOrdersMode::BuyExact);
            assert_eq!(spend_up_to_back, TakeOrdersMode::SpendUpTo);
            assert_eq!(spend_exact_back, TakeOrdersMode::SpendExact);
        }
    }
}
