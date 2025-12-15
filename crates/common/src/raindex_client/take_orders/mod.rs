mod request;
mod result;
mod selection;

#[cfg(all(test, not(target_family = "wasm")))]
mod e2e_tests;

pub use result::TakeOrdersCalldataResult;

use super::orders::{GetOrdersFilters, GetOrdersTokenFilter, RaindexOrder};
use super::{ChainIds, RaindexClient, RaindexError};
use crate::rpc_client::RpcClient;
use crate::take_orders::{build_take_orders_config_from_sell_simulation, MinReceiveMode};
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
        let req = request::parse_request(&sell_token, &buy_token, &sell_amount, min_receive_mode)?;

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
            selection::select_best_orderbook_simulation(candidates, req.sell_amount)?;

        let built = build_take_orders_config_from_sell_simulation(best_sim, req.min_receive_mode)?
            .ok_or(RaindexError::NoLiquidity)?;

        result::build_calldata_result(best_orderbook, built)
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
