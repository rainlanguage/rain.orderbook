use super::local_db::executor::JsCallbackExecutor;
use super::*;
use crate::local_db::{query::fetch_order_trades::LocalDbOrderTrade, LocalDb};
use crate::raindex_client::local_db::query::fetch_order_trades::fetch_order_trades;
use crate::raindex_client::local_db::query::fetch_order_trades_count::fetch_order_trades_count;
use crate::raindex_client::{
    orders::RaindexOrder,
    transactions::RaindexTransaction,
    vaults::{LocalTradeBalanceInfo, LocalTradeTokenInfo, RaindexVaultBalanceChange},
};
use alloy::primitives::{Address, Bytes, U256};
use rain_orderbook_subgraph_client::{
    types::{common::SgTrade, Id},
    SgPaginationArgs,
};
use std::str::FromStr;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_PAGE_SIZE: u16 = 100;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexTrade {
    id: Bytes,
    order_hash: Bytes,
    transaction: RaindexTransaction,
    input_vault_balance_change: RaindexVaultBalanceChange,
    output_vault_balance_change: RaindexVaultBalanceChange,
    timestamp: U256,
    orderbook: Address,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexTrade {
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn id(&self) -> String {
        self.id.to_string()
    }
    #[wasm_bindgen(getter = orderHash, unchecked_return_type = "Hex")]
    pub fn order_hash(&self) -> String {
        self.order_hash.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    #[wasm_bindgen(getter = inputVaultBalanceChange)]
    pub fn input_vault_balance_change(&self) -> RaindexVaultBalanceChange {
        self.input_vault_balance_change.clone()
    }
    #[wasm_bindgen(getter = outputVaultBalanceChange)]
    pub fn output_vault_balance_change(&self) -> RaindexVaultBalanceChange {
        self.output_vault_balance_change.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexTrade {
    pub fn id(&self) -> Bytes {
        self.id.clone()
    }
    pub fn order_hash(&self) -> Bytes {
        self.order_hash.clone()
    }
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    pub fn input_vault_balance_change(&self) -> RaindexVaultBalanceChange {
        self.input_vault_balance_change.clone()
    }
    pub fn output_vault_balance_change(&self) -> RaindexVaultBalanceChange {
        self.output_vault_balance_change.clone()
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
}

#[wasm_export]
impl RaindexOrder {
    /// Fetches trade history with optional time filtering
    ///
    /// Retrieves a chronological list of trades executed by an order within
    /// an optional time range.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await order.getTradesList();
    /// if (result.error) {
    ///   console.error("Cannot fetch trades:", result.error.readableMsg);
    ///   return;
    /// }
    /// const trades = result.value;
    /// // Do something with the trades
    /// ```
    #[wasm_export(
        js_name = "getTradesList",
        return_description = "Array of trade records with complete details",
        unchecked_return_type = "RaindexTrade[]"
    )]
    pub async fn get_trades_list(
        &self,
        #[wasm_export(
            js_name = "startTimestamp",
            param_description = "Optional start time filter (Unix timestamp in seconds)"
        )]
        start_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "endTimestamp",
            param_description = "Optional end time filter (Unix timestamp in seconds)"
        )]
        end_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "page",
            param_description = "Optional page number (defaults to 1)"
        )]
        page: Option<u16>,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let chain_id = self.chain_id();
        #[cfg(target_family = "wasm")]
        let orderbook = Address::from_str(&self.orderbook())?;
        #[cfg(not(target_family = "wasm"))]
        let orderbook = self.orderbook();

        if LocalDb::check_support(chain_id) {
            let raindex_client = self.get_raindex_client();
            if let Some(db_cb) = raindex_client.local_db_callback() {
                let exec = JsCallbackExecutor::from_ref(&db_cb);
                let order_hash = self.order_hash().to_string();
                let local_trades = fetch_order_trades(
                    &exec,
                    chain_id,
                    orderbook,
                    &order_hash,
                    start_timestamp,
                    end_timestamp,
                )
                .await?;

                let trades = local_trades
                    .into_iter()
                    .map(|trade| RaindexTrade::try_from_local_db_trade(chain_id, trade))
                    .collect::<Result<Vec<RaindexTrade>, RaindexError>>()?;

                return Ok(trades);
            }
        }

        let client = self.get_orderbook_client()?;
        let trades = client
            .order_trades_list(
                Id::new(self.id().to_string()),
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: DEFAULT_PAGE_SIZE,
                },
                start_timestamp,
                end_timestamp,
            )
            .await?;

        let trades = trades
            .into_iter()
            .map(|trade| RaindexTrade::try_from_sg_trade(self.chain_id(), trade))
            .collect::<Result<Vec<RaindexTrade>, RaindexError>>()?;
        Ok(trades)
    }

    /// Fetches detailed information for a specific trade
    ///
    /// Retrieves complete information about a single trade including vault changes
    /// and transaction details.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await order.getTradeDetail("0x1234567890abcdef1234567890abcdef12345678");
    /// if (result.error) {
    ///   console.error("Trade not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const trade = result.value;
    /// // Do something with the trade
    /// ```
    #[wasm_export(
        js_name = "getTradeDetail",
        return_description = "Complete trade information",
        unchecked_return_type = "RaindexTrade"
    )]
    pub async fn get_trade_detail_wasm_binding(
        &self,
        #[wasm_export(
            js_name = "tradeId",
            param_description = "Unique trade identifier",
            unchecked_param_type = "Hex"
        )]
        trade_id: String,
    ) -> Result<RaindexTrade, RaindexError> {
        let trade_id = Bytes::from_str(&trade_id)?;
        self.get_trade_detail(trade_id).await
    }

    /// Counts total trades for an order within a time range
    ///
    /// Efficiently counts the total number of trades executed by an order without
    /// fetching all trade details.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await order.getTradeCount();
    /// if (result.error) {
    ///   console.error("Cannot count trades:", result.error.readableMsg);
    ///   return;
    /// }
    /// const count = result.value;
    /// // Do something with the count
    /// ```
    #[wasm_export(
        js_name = "getTradeCount",
        return_description = "Total trade count as number",
        unchecked_return_type = "number"
    )]
    pub async fn get_trade_count(
        &self,
        #[wasm_export(
            js_name = "startTimestamp",
            param_description = "Optional start time filter (Unix timestamp in seconds)"
        )]
        start_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "endTimestamp",
            param_description = "Optional end time filter (Unix timestamp in seconds)"
        )]
        end_timestamp: Option<u64>,
    ) -> Result<u64, RaindexError> {
        let chain_id = self.chain_id();
        #[cfg(target_family = "wasm")]
        let orderbook = Address::from_str(&self.orderbook())?;
        #[cfg(not(target_family = "wasm"))]
        let orderbook = self.orderbook();

        if LocalDb::check_support(chain_id) {
            let raindex_client = self.get_raindex_client();
            if let Some(db_cb) = raindex_client.local_db_callback() {
                let exec = JsCallbackExecutor::from_ref(&db_cb);
                let order_hash = self.order_hash().to_string();
                let count = fetch_order_trades_count(
                    &exec,
                    chain_id,
                    orderbook,
                    &order_hash,
                    start_timestamp,
                    end_timestamp,
                )
                .await?;
                return Ok(count);
            }
        }

        let client = self.get_orderbook_client()?;
        let trades_count = client
            .order_trades_list_all(
                Id::new(self.id().to_string()),
                start_timestamp,
                end_timestamp,
            )
            .await?;
        Ok(trades_count.len() as u64)
    }
}
impl RaindexOrder {
    pub async fn get_trade_detail(&self, trade_id: Bytes) -> Result<RaindexTrade, RaindexError> {
        let client = self.get_orderbook_client()?;
        RaindexTrade::try_from_sg_trade(
            self.chain_id(),
            client
                .order_trade_detail(Id::new(trade_id.to_string()))
                .await?,
        )
    }
}

impl RaindexTrade {
    pub fn try_from_sg_trade(chain_id: u32, trade: SgTrade) -> Result<Self, RaindexError> {
        Ok(RaindexTrade {
            id: Bytes::from_str(&trade.id.0)?,
            order_hash: Bytes::from_str(&trade.order.order_hash.0)?,
            transaction: RaindexTransaction::try_from(trade.trade_event.transaction)?,
            input_vault_balance_change:
                RaindexVaultBalanceChange::try_from_sg_trade_balance_change(
                    chain_id,
                    trade.input_vault_balance_change,
                )?,
            output_vault_balance_change:
                RaindexVaultBalanceChange::try_from_sg_trade_balance_change(
                    chain_id,
                    trade.output_vault_balance_change,
                )?,
            timestamp: U256::from_str(&trade.timestamp.0)?,
            orderbook: Address::from_str(&trade.orderbook.id.0)?,
        })
    }

    pub(crate) fn try_from_local_db_trade(
        chain_id: u32,
        trade: LocalDbOrderTrade,
    ) -> Result<Self, RaindexError> {
        let orderbook = Address::from_str(&trade.orderbook_address)?;
        let transaction = RaindexTransaction::from_local_parts(
            &trade.transaction_hash,
            &trade.transaction_sender,
            trade.block_number,
            trade.block_timestamp,
        )?;

        let input_change = RaindexVaultBalanceChange::try_from_local_trade_side(
            chain_id,
            orderbook,
            &transaction,
            &trade.input_vault_id,
            LocalTradeTokenInfo {
                address: trade.input_token.clone(),
                name: trade.input_token_name.clone(),
                symbol: trade.input_token_symbol.clone(),
                decimals: trade.input_token_decimals,
            },
            LocalTradeBalanceInfo {
                delta: trade.input_delta.clone(),
                running_balance: trade.input_running_balance.clone(),
            },
            trade.block_timestamp,
        )?;

        let output_change = RaindexVaultBalanceChange::try_from_local_trade_side(
            chain_id,
            orderbook,
            &transaction,
            &trade.output_vault_id,
            LocalTradeTokenInfo {
                address: trade.output_token.clone(),
                name: trade.output_token_name.clone(),
                symbol: trade.output_token_symbol.clone(),
                decimals: trade.output_token_decimals,
            },
            LocalTradeBalanceInfo {
                delta: trade.output_delta.clone(),
                running_balance: trade.output_running_balance.clone(),
            },
            trade.block_timestamp,
        )?;

        Ok(RaindexTrade {
            id: Bytes::from_str(&trade.trade_id)?,
            order_hash: Bytes::from_str(&trade.order_hash)?,
            transaction,
            input_vault_balance_change: input_change,
            output_vault_balance_change: output_change,
            timestamp: U256::from(trade.block_timestamp),
            orderbook,
        })
    }
}

#[cfg(test)]
mod test_helpers {
    #[cfg(target_family = "wasm")]
    use super::*;
    #[cfg(not(target_family = "wasm"))]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::local_db::query::{
            fetch_order_trades::LocalDbOrderTrade, fetch_orders::LocalDbOrder,
            fetch_vault::LocalDbVault,
        };
        use crate::raindex_client::tests::{
            get_local_db_test_yaml, new_test_client_with_db_callback,
        };
        use alloy::primitives::{Address, Bytes};
        use rain_orderbook_subgraph_client::utils::float::{F1, F2, F3, NEG2};
        use serde_json::{self, json};
        use std::str::FromStr;
        use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::WasmEncodedResult;

        #[derive(Clone)]
        struct LocalTradeFixture {
            order: LocalDbOrder,
            input_vault: LocalDbVault,
            output_vault: LocalDbVault,
            trade: LocalDbOrderTrade,
            orderbook_address: String,
            order_hash: String,
            input_token: String,
            output_token: String,
        }

        fn build_local_trade_fixture(
            tx_hash: &str,
            trade_log_index: u64,
            trade_count: u64,
        ) -> LocalTradeFixture {
            const ORDERBOOK_ADDRESS: &str = "0x2f209e5b67a33b8fe96e28f24628df6da301c8eb";
            const ORDER_HASH: &str =
                "0x0000000000000000000000000000000000000000000000000000000000000abc";
            const OWNER: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            const INPUT_VAULT_ID: &str = "0x01";
            const OUTPUT_VAULT_ID: &str = "0x02";
            const INPUT_TOKEN: &str = "0x00000000000000000000000000000000000000aa";
            const OUTPUT_TOKEN: &str = "0x00000000000000000000000000000000000000bb";
            const INPUT_DELTA_HEX: &str =
                "0x0000000000000000000000000000000000000000000000000000000000000001";
            const INPUT_RUNNING_HEX: &str =
                "0x0000000000000000000000000000000000000000000000000000000000000003";
            const OUTPUT_DELTA_HEX: &str =
                "0x00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
            const OUTPUT_RUNNING_HEX: &str =
                "0x0000000000000000000000000000000000000000000000000000000000000001";

            let trade_id = format!(
                "0x{}{:016x}",
                tx_hash.trim_start_matches("0x"),
                trade_log_index
            )
            .to_lowercase();

            let order = LocalDbOrder {
                order_hash: ORDER_HASH.to_string(),
                owner: OWNER.to_string(),
                block_timestamp: 1_700_000_010,
                block_number: 123_456,
                orderbook_address: ORDERBOOK_ADDRESS.to_string(),
                order_bytes: "0x00000000000000000000000000000000000000000000000000000000000000ff"
                    .to_string(),
                transaction_hash: tx_hash.to_string(),
                inputs: Some(format!("0:{}:{}", INPUT_VAULT_ID, INPUT_TOKEN)),
                outputs: Some(format!("0:{}:{}", OUTPUT_VAULT_ID, OUTPUT_TOKEN)),
                trade_count,
                active: true,
                meta: Some("0x1234".to_string()),
            };

            let input_vault = LocalDbVault {
                vault_id: INPUT_VAULT_ID.to_string(),
                token: INPUT_TOKEN.to_string(),
                owner: OWNER.to_string(),
                orderbook_address: ORDERBOOK_ADDRESS.to_string(),
                token_name: "Token A".to_string(),
                token_symbol: "TKNA".to_string(),
                token_decimals: 18,
                balance: INPUT_RUNNING_HEX.to_string(),
                input_orders: Some(format!("0x01:{}:0", ORDER_HASH)),
                output_orders: None,
            };

            let output_vault = LocalDbVault {
                vault_id: OUTPUT_VAULT_ID.to_string(),
                token: OUTPUT_TOKEN.to_string(),
                owner: OWNER.to_string(),
                orderbook_address: ORDERBOOK_ADDRESS.to_string(),
                token_name: "Token B".to_string(),
                token_symbol: "TKNB".to_string(),
                token_decimals: 6,
                balance: OUTPUT_RUNNING_HEX.to_string(),
                input_orders: None,
                output_orders: Some(format!("0x01:{}:0", ORDER_HASH)),
            };

            let trade = LocalDbOrderTrade {
                trade_kind: "take".into(),
                orderbook_address: ORDERBOOK_ADDRESS.to_string(),
                order_hash: ORDER_HASH.to_string(),
                order_owner: OWNER.to_string(),
                order_nonce: "0".into(),
                transaction_hash: tx_hash.to_string(),
                log_index: trade_log_index,
                block_number: 123_460,
                block_timestamp: 1_700_000_000,
                transaction_sender: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                input_vault_id: INPUT_VAULT_ID.to_string(),
                input_token: INPUT_TOKEN.to_string(),
                input_token_name: Some("Token A".into()),
                input_token_symbol: Some("TKNA".into()),
                input_token_decimals: Some(18),
                input_delta: INPUT_DELTA_HEX.into(),
                input_running_balance: Some(INPUT_RUNNING_HEX.into()),
                output_vault_id: OUTPUT_VAULT_ID.to_string(),
                output_token: OUTPUT_TOKEN.to_string(),
                output_token_name: Some("Token B".into()),
                output_token_symbol: Some("TKNB".into()),
                output_token_decimals: Some(6),
                output_delta: OUTPUT_DELTA_HEX.into(),
                output_running_balance: Some(OUTPUT_RUNNING_HEX.into()),
                trade_id,
            };

            LocalTradeFixture {
                order,
                input_vault,
                output_vault,
                trade,
                orderbook_address: ORDERBOOK_ADDRESS.to_string(),
                order_hash: ORDER_HASH.to_string(),
                input_token: INPUT_TOKEN.to_string(),
                output_token: OUTPUT_TOKEN.to_string(),
            }
        }

        fn make_local_db_trades_callback(
            orders: Vec<LocalDbOrder>,
            vaults: Vec<LocalDbVault>,
            trades: Vec<LocalDbOrderTrade>,
            trade_count: u64,
        ) -> js_sys::Function {
            let orders_json = serde_json::to_string(&orders).unwrap();
            let orders_result = WasmEncodedResult::Success::<String> {
                value: orders_json,
                error: None,
            };
            let orders_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&orders_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let trades_json = serde_json::to_string(&trades).unwrap();
            let trades_result = WasmEncodedResult::Success::<String> {
                value: trades_json,
                error: None,
            };
            let trades_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&trades_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let trade_count_rows =
                serde_json::to_string(&vec![json!({ "trade_count": trade_count })]).unwrap();
            let trade_count_result = WasmEncodedResult::Success::<String> {
                value: trade_count_rows,
                error: None,
            };
            let trade_count_payload = js_sys::JSON::stringify(
                &serde_wasm_bindgen::to_value(&trade_count_result).unwrap(),
            )
            .unwrap()
            .as_string()
            .unwrap();

            let empty_result = WasmEncodedResult::Success::<String> {
                value: "[]".to_string(),
                error: None,
            };
            let empty_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&empty_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let mut vault_payloads: Vec<(String, String)> = Vec::new();
            for vault in vaults.into_iter() {
                let lookup = format!("'{}'", vault.vault_id);
                let json = serde_json::to_string(&vec![vault]).unwrap();
                let result = WasmEncodedResult::Success::<String> {
                    value: json,
                    error: None,
                };
                let payload =
                    js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&result).unwrap())
                        .unwrap()
                        .as_string()
                        .unwrap();
                vault_payloads.push((lookup, payload));
            }

            let callback = Closure::wrap(Box::new(move |sql: String| -> JsValue {
                if sql.contains("FROM order_events")
                    && sql.contains("GROUP_CONCAT(")
                    && sql.contains("ios.io_type = 'input'")
                {
                    return js_sys::JSON::parse(&orders_payload).unwrap();
                }

                if sql.contains("SELECT COUNT(*) AS trade_count") {
                    return js_sys::JSON::parse(&trade_count_payload).unwrap();
                }

                if sql.contains("AS trade_kind") {
                    return js_sys::JSON::parse(&trades_payload).unwrap();
                }

                if sql.contains("FLOAT_SUM(vd") {
                    for (needle, payload) in &vault_payloads {
                        if sql.contains(needle) {
                            return js_sys::JSON::parse(payload).unwrap();
                        }
                    }
                }

                js_sys::JSON::parse(&empty_payload).unwrap()
            }) as Box<dyn Fn(String) -> JsValue>);

            callback.into_js_value().dyn_into().unwrap()
        }

        #[wasm_bindgen_test]
        async fn test_get_trades_list_local_db_path() {
            let trade_log_index = 1u64;
            let tx_hash = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let fixture = build_local_trade_fixture(tx_hash, trade_log_index, 4);
            let trade_id = fixture.trade.trade_id.clone();

            let callback = make_local_db_trades_callback(
                vec![fixture.order.clone()],
                vec![fixture.input_vault.clone(), fixture.output_vault.clone()],
                vec![fixture.trade.clone()],
                4,
            );
            let client = new_test_client_with_db_callback(vec![get_local_db_test_yaml()], callback);

            let order = client
                .get_order_by_hash(
                    42161,
                    Address::from_str(&fixture.orderbook_address).unwrap(),
                    Bytes::from_str(&fixture.order_hash).unwrap(),
                )
                .await
                .unwrap();

            let trades = order.get_trades_list(None, None, None).await.unwrap();

            assert_eq!(trades.len(), 1);

            let trade = trades.first().unwrap();
            assert_eq!(trade.id(), trade_id);
            assert_eq!(trade.order_hash(), fixture.order_hash);
            assert_eq!(
                trade.orderbook().to_lowercase(),
                fixture.orderbook_address.to_lowercase()
            );

            let transaction = trade.transaction();
            assert_eq!(transaction.id(), tx_hash.to_string());
            assert_eq!(
                transaction.from().to_lowercase(),
                fixture.trade.transaction_sender.to_lowercase()
            );
            let block_number = transaction
                .block_number()
                .unwrap()
                .to_string(10)
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(block_number, fixture.trade.block_number.to_string());
            let timestamp = transaction
                .timestamp()
                .unwrap()
                .to_string(10)
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(timestamp, fixture.trade.block_timestamp.to_string());

            let trade_timestamp = trade
                .timestamp()
                .unwrap()
                .to_string(10)
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(trade_timestamp, fixture.trade.block_timestamp.to_string());

            let input_change = trade.input_vault_balance_change();
            assert!(input_change.amount().eq(F1).unwrap());
            assert!(input_change.new_balance().eq(F3).unwrap());
            assert!(input_change.old_balance().eq(F2).unwrap());
            assert_eq!(
                input_change.token().symbol(),
                fixture.trade.input_token_symbol.clone()
            );
            assert_eq!(
                input_change.token().address().to_lowercase(),
                fixture.input_token.to_lowercase()
            );

            let output_change = trade.output_vault_balance_change();
            assert!(output_change.amount().eq(NEG2).unwrap());
            assert!(output_change.new_balance().eq(F1).unwrap());
            assert!(output_change.old_balance().eq(F3).unwrap());
            assert_eq!(
                output_change.token().symbol(),
                fixture.trade.output_token_symbol.clone()
            );
            assert_eq!(
                output_change.token().address().to_lowercase(),
                fixture.output_token.to_lowercase()
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_trade_count_local_db_path() {
            let fixture = build_local_trade_fixture(
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                3,
                7,
            );

            let callback = make_local_db_trades_callback(
                vec![fixture.order.clone()],
                vec![fixture.input_vault.clone(), fixture.output_vault.clone()],
                vec![fixture.trade.clone()],
                7,
            );
            let client = new_test_client_with_db_callback(vec![get_local_db_test_yaml()], callback);

            let order = client
                .get_order_by_hash(
                    42161,
                    Address::from_str(&fixture.orderbook_address).unwrap(),
                    Bytes::from_str(&fixture.order_hash).unwrap(),
                )
                .await
                .unwrap();

            let count = order
                .get_trade_count(Some(1_699_999_900), Some(1_700_000_900))
                .await
                .unwrap();

            assert_eq!(count, 7);
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use alloy::primitives::Bytes;
        use httpmock::MockServer;
        use rain_math_float::Float;
        use rain_orderbook_subgraph_client::utils::float::*;
        use serde_json::{json, Value};

        fn get_order1_json() -> Value {
            json!(                        {
              "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
              "orderBytes": "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e1200000010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33",
              "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
              "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
              "outputs": [
                {
                  "id": "0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": Float::parse("0.987".to_string()).unwrap(),
                  "token": {
                    "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Staked FLR",
                    "symbol": "sFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                  },
                  "ordersAsOutput": [
                    {
                      "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
                      "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
                      "active": true
                    }
                  ],
                  "ordersAsInput": [],
                  "balanceChanges": []
                },
                {
                    "id": "0x0000000000000000000000000000000000000000",
                    "token": {
                      "id": "0x0000000000000000000000000000000000000000",
                      "address": "0x0000000000000000000000000000000000000000",
                      "name": "T1",
                      "symbol": "T1",
                      "decimals": "0"
                    },
                    "balance": F0,
                    "vaultId": "0",
                    "owner": "0x0000000000000000000000000000000000000000",
                    "ordersAsOutput": [],
                    "ordersAsInput": [],
                    "balanceChanges": [],
                    "orderbook": {
                      "id": "0x0000000000000000000000000000000000000000"
                    }
                  }
              ],
              "inputs": [
                {
                  "id": "0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac",
                  "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                  "vaultId": "75486334982066122983501547829219246999490818941767825330875804445439814023987",
                  "balance": Float::parse("0.79799".to_string()).unwrap(),
                  "token": {
                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "name": "Wrapped Flare",
                    "symbol": "WFLR",
                    "decimals": "18"
                  },
                  "orderbook": {
                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                  },
                  "ordersAsOutput": [],
                  "ordersAsInput": [
                    {
                      "id": "0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1",
                      "orderHash": "0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4",
                      "active": true
                    }
                  ],
                  "balanceChanges": []
                },
                {
                    "id": "0x0000000000000000000000000000000000000000",
                    "token": {
                      "id": "0x0000000000000000000000000000000000000000",
                      "address": "0x0000000000000000000000000000000000000000",
                      "name": "T1",
                      "symbol": "T1",
                      "decimals": "0"
                    },
                    "balance": F0,
                    "vaultId": "0",
                    "owner": "0x0000000000000000000000000000000000000000",
                    "ordersAsOutput": [],
                    "ordersAsInput": [],
                    "balanceChanges": [],
                    "orderbook": {
                      "id": "0x0000000000000000000000000000000000000000"
                    }
                  }
              ],
              "orderbook": {
                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
              },
              "active": true,
              "timestampAdded": "1739448802",
              "meta": "0xff0a89c674ee7874a3005902252f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a616d743a203130302c0a696f3a2063616c6c3c323e28293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a63616c6c3c333e28292c0a3a656e7375726528657175616c2d746f286f75747075742d7661756c742d64656372656173652829203130302920226d7573742074616b652066756c6c20616d6f756e7422293b0a0a2f2a20322e206765742d696f2d726174696f2d6e6f77202a2f200a656c61707365643a2063616c6c3c343e28292c0a696f3a2073617475726174696e672d73756228302e3031373733353620646976286d756c28656c61707365642073756228302e3031373733353620302e30313733383434292920363029293b0a0a2f2a20332e206f6e652d73686f74202a2f200a3a656e737572652869732d7a65726f286765742868617368286f726465722d68617368282920226861732d657865637574656422292929202268617320657865637574656422292c0a3a7365742868617368286f726465722d68617368282920226861732d657865637574656422292031293b0a0a2f2a20342e206765742d656c6170736564202a2f200a5f3a20737562286e6f772829206765742868617368286f726465722d68617368282920226465706c6f792d74696d65222929293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d",
              "addEvents": [
                {
                  "transaction": {
                    "id": "0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af",
                    "from": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                    "blockNumber": "37432554",
                    "timestamp": "1739448802"
                  }
                }
              ],
              "trades": [],
              "removeEvents": []
            })
        }

        fn get_single_trade_json() -> Value {
            json!(              {
              "id": "0x0123",
              "tradeEvent": {
                "transaction": {
                  "id": "0x0123",
                  "from": "0x0000000000000000000000000000000000000000",
                  "blockNumber": "0",
                  "timestamp": "0"
                },
                "sender": "sender1"
              },
              "outputVaultBalanceChange": {
                "id": "0x0123",
                "__typename": "TradeVaultBalanceChange",
                "amount": NEG2,
                "newVaultBalance": F0,
                "oldVaultBalance": F0,
                "vault": {
                  "id": "0x0123",
                  "vaultId": "0x0123",
                  "token": {
                    "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Staked FLR",
                    "symbol": "sFLR",
                    "decimals": "18"
                  }
                },
                "timestamp": "1700000000",
                "transaction": {
                  "id": "0x0123",
                  "from": "0x0000000000000000000000000000000000000000",
                  "blockNumber": "0",
                  "timestamp": "1700000000"
                },
                "orderbook": {
                  "id": "0x1234567890abcdef1234567890abcdef12345678"
                }
              },
              "order": {
                "id": "0x0123",
                "orderHash": "0x0123"
              },
              "inputVaultBalanceChange": {
                "id": "0x0123",
                "__typename": "TradeVaultBalanceChange",
                "amount": F1,
                "newVaultBalance": F0,
                "oldVaultBalance": F0,
                "vault": {
                  "id": "0x0123",
                  "vaultId": "0x0123",
                  "token": {
                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "name": "Wrapped Flare",
                    "symbol": "WFLR",
                    "decimals": "18"
                  }
                },
                "timestamp": "1700000000",
                "transaction": {
                  "id": "0x0123",
                  "from": "0x0000000000000000000000000000000000000000",
                  "blockNumber": "0",
                  "timestamp": "1700000000"
                },
                "orderbook": {
                  "id": "0x1234567890abcdef1234567890abcdef12345678"
                }
              },
              "timestamp": "0",
              "orderbook": {
                "id": "0x1234567890abcdef1234567890abcdef12345678"
              }
            })
        }
        fn get_trades_json() -> Value {
            json!([
                get_single_trade_json(),
              {
                "id": "0x0234",
                "tradeEvent": {
                  "transaction": {
                    "id": "0x0234",
                    "from": "0x0000000000000000000000000000000000000001",
                    "blockNumber": "0",
                    "timestamp": "0"
                  },
                  "sender": "sender2"
                },
                "outputVaultBalanceChange": {
                  "id": "0x0234",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": NEG5,
                  "newVaultBalance": F0,
                  "oldVaultBalance": F0,
                  "vault": {
                    "id": "0x0234",
                    "vaultId": "0x0234",
                    "token": {
                      "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "name": "Staked FLR",
                      "symbol": "sFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "1700086400",
                  "transaction": {
                    "id": "0x0234",
                    "from": "0x0000000000000000000000000000000000000001",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": "0x1234567890abcdef1234567890abcdef12345679"
                  }
                },
                "order": {
                  "id": "0x0234",
                  "orderHash": "0x0234"
                },
                "inputVaultBalanceChange": {
                  "id": "0x0234",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": F2,
                  "newVaultBalance": F0,
                  "oldVaultBalance": F0,
                  "vault": {
                    "id": "0x0234",
                    "vaultId": "0x0234",
                    "token": {
                      "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "name": "Wrapped Flare",
                      "symbol": "WFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "0",
                  "transaction": {
                    "id": "0x0234",
                    "from": "0x0000000000000000000000000000000000000005",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": "0x1234567890abcdef1234567890abcdef12345679"
                  }
                },
                "timestamp": "1700086400",
                "orderbook": {
                  "id": "0x1234567890abcdef1234567890abcdef12345679"
                }
              }
            ])
        }

        #[tokio::test]
        async fn test_get_order_trades_list() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderTradesListQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "trades": get_trades_json()
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderTradesListQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "trades": []
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderDetailByHashQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let trades = order.get_trades_list(None, None, None).await.unwrap();
            assert_eq!(trades.len(), 2);

            let trade1 = &trades[0].clone();
            assert_eq!(trade1.id(), Bytes::from_str("0x0123").unwrap());
            assert_eq!(
                trade1.transaction().id(),
                Bytes::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade1.transaction().from(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(trade1.transaction().block_number(), U256::ZERO);
            assert_eq!(trade1.transaction().timestamp(), U256::ZERO);
            // assert_eq!(trade1.trade_event.sender.0, "sender1");

            assert!(trade1
                .output_vault_balance_change()
                .amount()
                .eq(NEG2)
                .unwrap());
            assert!(trade1
                .output_vault_balance_change()
                .new_balance()
                .eq(F0)
                .unwrap());
            assert!(trade1
                .output_vault_balance_change()
                .old_balance()
                .eq(F0)
                .unwrap());

            assert_eq!(
                trade1.output_vault_balance_change().vault_id(),
                U256::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade1.output_vault_balance_change().token().id(),
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade1.output_vault_balance_change().token().address(),
                Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap()
            );
            assert_eq!(
                trade1.output_vault_balance_change().token().name(),
                Some("Staked FLR".to_string())
            );
            assert_eq!(
                trade1.output_vault_balance_change().token().symbol(),
                Some("sFLR".to_string())
            );
            assert_eq!(trade1.output_vault_balance_change().token().decimals(), 18);
            assert_eq!(
                trade1.output_vault_balance_change().timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(
                trade1.output_vault_balance_change().transaction().id(),
                Bytes::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade1.output_vault_balance_change().transaction().from(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(
                trade1
                    .output_vault_balance_change()
                    .transaction()
                    .block_number(),
                U256::ZERO
            );
            assert_eq!(
                trade1
                    .output_vault_balance_change()
                    .transaction()
                    .timestamp(),
                U256::from_str("1700000000").unwrap()
            );

            assert!(trade1.input_vault_balance_change().amount().eq(F1).unwrap());
            assert!(trade1
                .input_vault_balance_change()
                .new_balance()
                .eq(F0)
                .unwrap());
            assert!(trade1
                .input_vault_balance_change()
                .old_balance()
                .eq(F0)
                .unwrap());

            assert_eq!(
                trade1.input_vault_balance_change().vault_id(),
                U256::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade1.input_vault_balance_change().token().id(),
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade1.input_vault_balance_change().token().address(),
                Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
            );
            assert_eq!(
                trade1.input_vault_balance_change().token().name(),
                Some("Wrapped Flare".to_string())
            );
            assert_eq!(
                trade1.input_vault_balance_change().token().symbol(),
                Some("WFLR".to_string())
            );
            assert_eq!(trade1.input_vault_balance_change().token().decimals(), 18);
            assert_eq!(
                trade1.input_vault_balance_change().timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(
                trade1.input_vault_balance_change().transaction().id(),
                Bytes::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade1
                    .input_vault_balance_change()
                    .transaction()
                    .block_number(),
                U256::ZERO
            );
            assert_eq!(
                trade1
                    .input_vault_balance_change()
                    .transaction()
                    .timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(trade1.timestamp(), U256::ZERO);
            assert_eq!(
                trade1.orderbook(),
                Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap()
            );
            assert_eq!(trade1.order_hash(), Bytes::from_str("0x0123").unwrap());

            let trade2 = trades[1].clone();
            assert_eq!(trade2.id(), Bytes::from_str("0x0234").unwrap());
        }

        #[tokio::test]
        async fn test_get_order_trade_detail() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderTradeDetailQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "trade": get_single_trade_json()
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderDetailByHashQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let trade = order
                .get_trade_detail(Bytes::from_str("0x0123").unwrap())
                .await
                .unwrap();
            assert_eq!(trade.id(), Bytes::from_str("0x0123").unwrap());
            assert_eq!(trade.transaction().id(), Bytes::from_str("0x0123").unwrap());
            assert_eq!(
                trade.transaction().from(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(trade.transaction().block_number(), U256::ZERO);
            assert_eq!(trade.transaction().timestamp(), U256::ZERO);
            // assert_eq!(trade.trade_event.sender.0, "sender1");

            assert!(trade
                .output_vault_balance_change()
                .amount()
                .eq(NEG2)
                .unwrap());
            assert!(trade
                .output_vault_balance_change()
                .new_balance()
                .eq(F0)
                .unwrap());
            assert!(trade
                .output_vault_balance_change()
                .old_balance()
                .eq(F0)
                .unwrap());

            assert_eq!(
                trade.output_vault_balance_change().vault_id(),
                U256::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade.output_vault_balance_change().token().id(),
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade.output_vault_balance_change().token().address(),
                Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap()
            );
            assert_eq!(
                trade.output_vault_balance_change().token().name(),
                Some("Staked FLR".to_string())
            );
            assert_eq!(
                trade.output_vault_balance_change().token().symbol(),
                Some("sFLR".to_string())
            );
            assert_eq!(trade.output_vault_balance_change().token().decimals(), 18);
            assert_eq!(
                trade.output_vault_balance_change().timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(
                trade.output_vault_balance_change().transaction().id(),
                Bytes::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade
                    .output_vault_balance_change()
                    .transaction()
                    .block_number(),
                U256::ZERO
            );
            assert_eq!(
                trade
                    .output_vault_balance_change()
                    .transaction()
                    .timestamp(),
                U256::from_str("1700000000").unwrap()
            );

            assert!(trade.input_vault_balance_change().amount().eq(F1).unwrap());
            assert!(trade
                .input_vault_balance_change()
                .new_balance()
                .eq(F0)
                .unwrap());
            assert!(trade
                .input_vault_balance_change()
                .old_balance()
                .eq(F0)
                .unwrap());

            assert_eq!(
                trade.input_vault_balance_change().vault_id(),
                U256::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade.input_vault_balance_change().token().id(),
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade.input_vault_balance_change().token().address(),
                Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
            );
            assert_eq!(
                trade.input_vault_balance_change().token().name(),
                Some("Wrapped Flare".to_string())
            );
            assert_eq!(
                trade.input_vault_balance_change().token().symbol(),
                Some("WFLR".to_string())
            );
            assert_eq!(trade.input_vault_balance_change().token().decimals(), 18);
            assert_eq!(
                trade.input_vault_balance_change().timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(
                trade.input_vault_balance_change().transaction().id(),
                Bytes::from_str("0x0123").unwrap()
            );
            assert_eq!(
                trade.input_vault_balance_change().transaction().from(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(
                trade.input_vault_balance_change().transaction().timestamp(),
                U256::from_str("1700000000").unwrap()
            );
            assert_eq!(trade.timestamp(), U256::ZERO);
            assert_eq!(
                trade.orderbook(),
                Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap()
            );
            assert_eq!(trade.order_hash(), Bytes::from_str("0x0123").unwrap());
        }

        #[tokio::test]
        async fn test_get_order_trades_count() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "trades": get_trades_json()
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOrderDetailByHashQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [get_order1_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();
            let order = raindex_client
                .get_order_by_hash(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let count = order.get_trade_count(None, None).await.unwrap();
            assert_eq!(count, 2);
        }
    }
}
