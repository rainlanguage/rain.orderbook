use super::orders::{GetOrdersFilters, GetOrdersTokenFilter};
use super::*;
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_order_candidates_for_pair, build_take_orders_config_from_sell_simulation,
    simulate_sell_over_candidates, MinReceiveMode,
};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use std::ops::Div;
use std::str::FromStr;

/// Combined result for generating takeOrders3 calldata and price info.
///
/// `calldata` can be sent directly as transaction data; `effective_price` and
/// `prices` provide blended and per-leg prices (sell per 1 buy).
#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersCalldataResult {
    /// ABI-encoded calldata for `IOrderBookV5.takeOrders3` (hex in JS)
    #[tsify(type = "Hex")]
    pub calldata: Bytes,
    /// Blended sell per 1 buy (totalSell / totalBuy)
    #[tsify(type = "Hex")]
    pub effective_price: Float,
    /// Per-leg prices (sell per 1 buy), sorted best (cheapest) to worst
    #[tsify(type = "Hex[]")]
    pub prices: Vec<Float>,
}
impl_wasm_traits!(TakeOrdersCalldataResult);

#[wasm_export]
impl RaindexClient {
    /// Generates ABI-encoded calldata for `takeOrders3()` using an exact-in builder.
    ///
    /// Discovers orders for the given `(chainId, sellToken, buyToken)` pair, simulates
    /// how much `buyToken` is received for the exact `sellAmount` budget, applies the
    /// `minReceiveMode` policy, and returns both calldata and price information.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getTakeOrdersCalldata(
    ///   137,                 // chainId
    ///   "0xSELL...",         // sellToken
    ///   "0xBUY...",          // buyToken
    ///   sellAmountFloat,     // exact sell amount as Float hex string
    ///   "partial",           // or "exact"
    /// );
    /// if (result.error) {
    ///   console.error("Failed to build takeOrders calldata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const { calldata, effectivePrice, prices } = result.value;
    /// // Use calldata for transaction, show prices in UI, etc.
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
            js_name = "sellAmount",
            param_description = "Exact sell amount as a Float hex string in sellToken units",
            unchecked_param_type = "Hex"
        )]
        sell_amount: String,
        #[wasm_export(
            js_name = "minReceiveMode",
            param_description = "Minimum receive policy: partial or exact"
        )]
        min_receive_mode: MinReceiveMode,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let sell_token_addr = Address::from_str(&sell_token)?;
        let buy_token_addr = Address::from_str(&buy_token)?;
        let sell_amount_float = Float::parse(sell_amount)?;

        let filters = GetOrdersFilters {
            owners: vec![],
            active: Some(true),
            order_hash: None,
            tokens: Some(GetOrdersTokenFilter {
                inputs: Some(vec![sell_token_addr]),
                outputs: Some(vec![buy_token_addr]),
            }),
        };

        let orders = self
            .get_orders(Some(ChainIds(vec![chain_id])), Some(filters), None)
            .await?;

        if orders.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let rpc_urls = self.get_rpc_urls_for_chain(chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls)?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates = build_take_order_candidates_for_pair(
            &orders,
            sell_token_addr,
            buy_token_addr,
            Some(block_number),
            None,
        )
        .await?;

        if candidates.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let sim = simulate_sell_over_candidates(candidates, sell_amount_float)?;

        if sim.legs.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let built = build_take_orders_config_from_sell_simulation(sim, min_receive_mode)?
            .ok_or(RaindexError::NoLiquidity)?;

        let calldata_bytes = takeOrders3Call {
            config: built.config,
        }
        .abi_encode();
        let calldata = Bytes::copy_from_slice(&calldata_bytes);

        let zero = Float::zero()?;
        let effective_price = if built.sim.total_buy_amount.gt(zero)? {
            built
                .sim
                .total_sell_amount
                .div(built.sim.total_buy_amount)?
        } else {
            zero
        };

        let prices: Vec<Float> = built
            .sim
            .legs
            .iter()
            .map(|leg| leg.candidate.ratio)
            .collect();

        Ok(TakeOrdersCalldataResult {
            calldata,
            effective_price,
            prices,
        })
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
