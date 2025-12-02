use super::super::orders::{GetOrdersFilters, OrdersDataSource, RaindexOrder};
use super::{LocalDb, RaindexError};
use crate::local_db::query::fetch_vaults::LocalDbVault;
use crate::local_db::query::LocalDbQueryError;
use crate::local_db::{query::fetch_orders::FetchOrdersArgs, OrderbookIdentifier};
use crate::raindex_client::local_db::query::fetch_orders::fetch_orders;
use crate::raindex_client::RaindexClient;
use alloy::primitives::B256;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::from_str;
use std::rc::Rc;

pub struct LocalDbOrders<'a> {
    pub(crate) db: &'a LocalDb,
    pub(crate) client: Rc<RaindexClient>,
}

impl<'a> LocalDbOrders<'a> {
    pub(crate) fn new(db: &'a LocalDb, client: Rc<RaindexClient>) -> Self {
        Self { db, client }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalDbOrderIo {
    #[serde(rename = "ioIndex")]
    io_index: usize,
    vault: LocalDbVault,
}

pub(crate) fn parse_io_vaults(
    label: &str,
    payload: &Option<String>,
) -> Result<Vec<LocalDbVault>, RaindexError> {
    let Some(raw) = payload else {
        return Ok(Vec::new());
    };

    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Vec<Option<LocalDbOrderIo>> = from_str(raw).map_err(|err| {
        RaindexError::from(LocalDbQueryError::deserialization(format!(
            "Failed to decode {label} payload: {err}"
        )))
    })?;

    let mut ios: Vec<LocalDbOrderIo> = parsed.into_iter().flatten().collect();
    ios.sort_by_key(|io| io.io_index);

    Ok(ios.into_iter().map(|io| io.vault).collect())
}

#[async_trait(?Send)]
impl OrdersDataSource for LocalDbOrders<'_> {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetOrdersFilters,
        _page: Option<u16>,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let mut fetch_args = FetchOrdersArgs::from(filters.clone());
        if let Some(ids) = chain_ids {
            if !ids.is_empty() {
                fetch_args.chain_ids = ids;
            }
        }

        let local_db_orders = fetch_orders(self.db, fetch_args).await?;
        let mut orders: Vec<RaindexOrder> = Vec::with_capacity(local_db_orders.len());
        let client = Rc::clone(&self.client);

        for local_db_order in local_db_orders {
            let inputs = parse_io_vaults("inputs", &local_db_order.inputs)?;
            let outputs = parse_io_vaults("outputs", &local_db_order.outputs)?;
            let order = RaindexOrder::from_local_db_order(
                Rc::clone(&client),
                local_db_order,
                inputs,
                outputs,
            )?;
            orders.push(order);
        }

        Ok(orders)
    }

    async fn get_by_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
    ) -> Result<Option<RaindexOrder>, RaindexError> {
        let fetch_args = FetchOrdersArgs {
            chain_ids: vec![ob_id.chain_id],
            orderbook_addresses: vec![ob_id.orderbook_address],
            order_hash: Some(*order_hash),
            ..FetchOrdersArgs::default()
        };

        let local_db_orders = fetch_orders(self.db, fetch_args).await?;
        let client = Rc::clone(&self.client);

        if let Some(local_db_order) = local_db_orders.into_iter().next() {
            let inputs = parse_io_vaults("inputs", &local_db_order.inputs)?;
            let outputs = parse_io_vaults("outputs", &local_db_order.outputs)?;
            let order = RaindexOrder::from_local_db_order(client, local_db_order, inputs, outputs)?;

            return Ok(Some(order));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use std::str::FromStr;

        use super::*;
        use crate::local_db::query::{fetch_orders::LocalDbOrder, fetch_vaults::LocalDbVault};
        use crate::raindex_client::tests::{
            get_local_db_test_yaml, new_test_client_with_db_callback,
        };
        use crate::raindex_client::ChainIds;
        use alloy::primitives::{address, b256, bytes, Bytes, U256};
        use serde_json::{self, json};
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::*;

        fn make_local_db_callback(orders: Vec<LocalDbOrder>) -> js_sys::Function {
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

            let empty_result = WasmEncodedResult::Success::<String> {
                value: "[]".to_string(),
                error: None,
            };
            let empty_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&empty_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let callback =
                Closure::wrap(Box::new(move |sql: String, _params: JsValue| -> JsValue {
                    if sql.contains("FROM order_events") && sql.contains("json_group_array") {
                        return js_sys::JSON::parse(&orders_payload).unwrap();
                    }

                    js_sys::JSON::parse(&empty_payload).unwrap()
                })
                    as Box<dyn Fn(String, JsValue) -> JsValue>);

            callback.into_js_value().dyn_into().unwrap()
        }

        #[wasm_bindgen_test]
        async fn test_get_orders_local_db_callback_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let order_hash_str = order_hash.to_string();
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ff");
            let order_bytes_str = order_bytes.to_string();
            let transaction_hash =
                b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let meta = Bytes::from_str("0x1234").unwrap();
            let meta_str = meta.to_string();
            let input_vault_id = U256::from_str("0x0a").unwrap();
            let output_vault_id = U256::from_str("0x0b").unwrap();
            let input_token = address!("0x00000000000000000000000000000000000000aa");
            let output_token = address!("0x00000000000000000000000000000000000000bb");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let input_vault = LocalDbVault {
                chain_id: 1,
                vault_id: input_vault_id,
                token: input_token,
                owner,
                orderbook_address,
                token_name: "Token A".to_string(),
                token_symbol: "TKNA".to_string(),
                token_decimals: 18,
                balance: "0x000000000000000000000000000000000000000000000000000000000000000a"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let output_vault = LocalDbVault {
                chain_id: 1,
                vault_id: output_vault_id,
                token: output_token,
                owner,
                orderbook_address,
                token_name: "Token B".to_string(),
                token_symbol: "TKNB".to_string(),
                token_decimals: 6,
                balance: "0x0000000000000000000000000000000000000000000000000000000000000005"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let inputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": input_vault.chain_id,
                    "vaultId": input_vault.vault_id,
                    "token": input_vault.token,
                    "owner": input_vault.owner,
                    "orderbookAddress": input_vault.orderbook_address,
                    "tokenName": input_vault.token_name,
                    "tokenSymbol": input_vault.token_symbol,
                    "tokenDecimals": input_vault.token_decimals,
                    "balance": input_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let outputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": output_vault.chain_id,
                    "vaultId": output_vault.vault_id,
                    "token": output_vault.token,
                    "owner": output_vault.owner,
                    "orderbookAddress": output_vault.orderbook_address,
                    "tokenName": output_vault.token_name,
                    "tokenSymbol": output_vault.token_symbol,
                    "tokenDecimals": output_vault.token_decimals,
                    "balance": output_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let local_order = LocalDbOrder {
                chain_id: 42161,
                order_hash: order_hash.clone(),
                owner,
                block_timestamp: 123456,
                block_number: 654321,
                orderbook_address,
                order_bytes: order_bytes,
                transaction_hash: transaction_hash.clone(),
                inputs: Some(inputs_json.to_string()),
                outputs: Some(outputs_json.to_string()),
                trade_count: 7,
                active: true,
                meta: Some(meta.clone()),
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);

            let client = new_test_client_with_db_callback(vec![get_local_db_test_yaml()], callback);

            let orders = client
                .get_orders(Some(ChainIds(vec![42161])), None, None)
                .await
                .expect("local db query should succeed");

            assert_eq!(orders.len(), 1);

            let order = &orders[0];
            assert_eq!(order.chain_id(), 42161);
            assert_eq!(order.order_hash(), order_hash_str);
            assert_eq!(order.order_bytes(), order_bytes_str);
            assert_eq!(
                order.owner().to_lowercase(),
                owner.to_string().to_lowercase()
            );
            assert!(order.active());
            assert_eq!(order.trades_count(), local_order.trade_count as u16);
            assert_eq!(order.meta(), Some(meta_str));
            assert_eq!(order.orderbook(), orderbook_address.to_string());
            assert!(order.transaction().is_none());

            let timestamp = order.timestamp_added().unwrap();
            let timestamp_str = timestamp
                .to_string(10)
                .expect("timestamp to_string should succeed")
                .as_string()
                .expect("timestamp string conversion should succeed");
            assert_eq!(timestamp_str, local_order.block_timestamp.to_string());

            let input_vaults = order.inputs_list().items();
            assert_eq!(input_vaults.len(), 1);
            assert_eq!(
                input_vaults[0].token().symbol(),
                Some(input_vault.token_symbol.clone())
            );
            assert_eq!(
                input_vaults[0].orderbook(),
                input_vault.orderbook_address.to_string()
            );

            let output_vaults = order.outputs_list().items();
            assert_eq!(output_vaults.len(), 1);
            assert_eq!(
                output_vaults[0].token().symbol(),
                Some(output_vault.token_symbol.clone())
            );
            assert_eq!(
                output_vaults[0].orderbook(),
                output_vault.orderbook_address.to_string()
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_order_by_hash_local_db_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let transaction_hash =
                b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ff");
            let meta = Bytes::from_str("0x1234").unwrap();
            let input_vault_id = U256::from_str("0x0a").unwrap();
            let output_vault_id = U256::from_str("0x0b").unwrap();
            let input_token = address!("0x00000000000000000000000000000000000000aa");
            let output_token = address!("0x00000000000000000000000000000000000000bb");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let input_vault = LocalDbVault {
                chain_id: 1,
                vault_id: input_vault_id,
                token: input_token,
                owner,
                orderbook_address,
                token_name: "Token A".to_string(),
                token_symbol: "TKNA".to_string(),
                token_decimals: 18,
                balance: "0x000000000000000000000000000000000000000000000000000000000000000a"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let output_vault = LocalDbVault {
                chain_id: 1,
                vault_id: output_vault_id,
                token: output_token,
                owner,
                orderbook_address,
                token_name: "Token B".to_string(),
                token_symbol: "TKNB".to_string(),
                token_decimals: 6,
                balance: "0x0000000000000000000000000000000000000000000000000000000000000005"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let inputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": input_vault.chain_id,
                    "vaultId": input_vault.vault_id,
                    "token": input_vault.token,
                    "owner": input_vault.owner,
                    "orderbookAddress": input_vault.orderbook_address,
                    "tokenName": input_vault.token_name,
                    "tokenSymbol": input_vault.token_symbol,
                    "tokenDecimals": input_vault.token_decimals,
                    "balance": input_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let outputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": output_vault.chain_id,
                    "vaultId": output_vault.vault_id,
                    "token": output_vault.token,
                    "owner": output_vault.owner,
                    "orderbookAddress": output_vault.orderbook_address,
                    "tokenName": output_vault.token_name,
                    "tokenSymbol": output_vault.token_symbol,
                    "tokenDecimals": output_vault.token_decimals,
                    "balance": output_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let local_order = LocalDbOrder {
                chain_id: 42161,
                order_hash,
                owner,
                block_timestamp: 123456,
                block_number: 654321,
                orderbook_address,
                order_bytes: order_bytes.clone(),
                transaction_hash: transaction_hash.clone(),
                inputs: Some(inputs_json.to_string()),
                outputs: Some(outputs_json.to_string()),
                trade_count: 3,
                active: true,
                meta: Some(meta.clone()),
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);

            let client = new_test_client_with_db_callback(vec![get_local_db_test_yaml()], callback);

            let order = client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(42161, orderbook_address),
                    order_hash.clone(),
                )
                .await
                .expect("local db order fetch should succeed");

            assert_eq!(order.chain_id(), 42161);
            assert_eq!(order.order_hash(), order_hash.to_string());
            assert_eq!(order.order_bytes(), order_bytes.to_string());
            assert_eq!(
                order.owner().to_lowercase(),
                owner.to_string().to_lowercase()
            );
            assert!(order.active());
            assert_eq!(order.trades_count(), local_order.trade_count as u16);
            assert_eq!(order.meta(), Some(meta.to_string()));
            assert_eq!(order.orderbook(), orderbook_address.to_string());
        }
    }
}
