use super::*;

// Import SQL files using include_str! macro
const CREATE_TABLES_SQL: &str = include_str!("sql/create_tables.sql");
const FETCH_ALL_TABLES_SQL: &str = include_str!("sql/fetch_all_tables.sql");
const ACTIVE_ORDERS_SQL: &str = include_str!("sql/get_active_orders.sql");
const ORDER_TRADES_SQL: &str = include_str!("sql/get_order_trades.sql");
const ORDER_VAULTS_VOLUME_SQL: &str = include_str!("sql/get_order_vaults_volume.sql");
const VAULT_BALANCE_HISTORY_SQL: &str = include_str!("sql/get_vault_balance_history.sql");
const GET_ALL_VAULTS_SQL: &str = include_str!("sql/get_all_vaults.sql");
const CLEAR_TABLES_SQL: &str = include_str!("sql/clear_tables.sql");

pub fn get_create_tables_query() -> String {
    CREATE_TABLES_SQL.to_string()
}

pub fn get_fetch_all_tables_query() -> String {
    FETCH_ALL_TABLES_SQL.to_string()
}

pub fn get_active_orders_query() -> String {
    ACTIVE_ORDERS_SQL.to_string()
}

pub fn get_order_trades_query(order_hash: &str) -> String {
    ORDER_TRADES_SQL.replace("?", &format!("'{}'", order_hash))
}

pub fn get_order_vaults_volume_query(order_hash: &str) -> String {
    ORDER_VAULTS_VOLUME_SQL.replace("?", &format!("'{}'", order_hash))
}

pub fn get_vault_balance_history_query(vault_id: &str, token: &str) -> String {
    VAULT_BALANCE_HISTORY_SQL
        .replace("?vault_id", &format!("'{}'", vault_id))
        .replace("?token", &format!("'{}'", token))
}

pub fn get_all_vaults_query() -> String {
    GET_ALL_VAULTS_SQL.to_string()
}

pub fn get_clear_tables_query() -> String {
    CLEAR_TABLES_SQL.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct TableResponse {
    pub name: String,
}
impl_wasm_traits!(TableResponse);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ActiveOrderResponse {
    #[serde(alias = "order_hash")]
    pub order_hash: String,
    pub owner: String,
    #[serde(alias = "creation_time")]
    pub creation_time: u64,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    #[serde(alias = "trade_count")]
    pub trade_count: u32,
}
impl_wasm_traits!(ActiveOrderResponse);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderTradeResponse {
    #[serde(alias = "trade_date")]
    pub trade_date: String,
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "input_amount")]
    pub input_amount: String,
    #[serde(alias = "output_amount")]
    pub output_amount: String,
}
impl_wasm_traits!(OrderTradeResponse);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderVaultVolumeResponse {
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub token: String,
    pub decimals: u8,
    #[serde(alias = "total_in")]
    pub total_in: f64,
    #[serde(alias = "total_out")]
    pub total_out: f64,
    #[serde(alias = "net_volume")]
    pub net_volume: f64,
    #[serde(alias = "total_volume")]
    pub total_volume: f64,
}
impl_wasm_traits!(OrderVaultVolumeResponse);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct VaultBalanceHistoryResponse {
    pub date: Option<String>,
    #[serde(alias = "tx_hash")]
    pub tx_hash: String,
    #[serde(alias = "balance_change")]
    pub balance_change: String,
    #[serde(alias = "balance_change_type")]
    pub balance_change_type: String,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "log_index")]
    pub log_index: u64,
}
impl_wasm_traits!(VaultBalanceHistoryResponse);

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AllVaultsResponse {
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub token: String,
    pub decimals: Option<u8>,
    pub owner: String,
    pub balance: f64,
    #[serde(alias = "input_order_hashes")]
    pub input_order_hashes: Option<String>,
    #[serde(alias = "output_order_hashes")]
    pub output_order_hashes: Option<String>,
}
impl_wasm_traits!(AllVaultsResponse);

/// Helper function to split SQL statements by semicolons
fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|stmt| stmt.trim())
        .filter(|stmt| !stmt.is_empty())
        .map(|stmt| stmt.to_string())
        .collect()
}

/// Execute a query that returns data and deserialize the result into the specified type
pub async fn execute_query_with_callback<T>(
    callback: &js_sys::Function,
    sql: &str,
) -> Result<T, RaindexError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let statements = split_sql_statements(sql);

    // For queries that return data, we typically expect a single statement
    // But we'll handle multiple statements by executing the last one that returns data
    let mut final_result = None;

    for statement in statements {
        let result = callback
            .call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(&statement),
            )
            .map_err(|e| RaindexError::CustomError(format!("Callback error: {:?}", e)))?;

        let promise = js_sys::Promise::resolve(&result);
        let future = wasm_bindgen_futures::JsFuture::from(promise);

        let result = future
            .await
            .map_err(|e| RaindexError::CustomError(format!("Promise error: {:?}", e)))?;

        // Handle the result as an object with either error or value properties
        if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
            // Check for error property first
            let error_prop = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("error"));
            if let Ok(error_val) = error_prop {
                if !error_val.is_undefined() {
                    // Try to get the readableMsg from the error object
                    if let Ok(error_obj) = error_val.dyn_into::<js_sys::Object>() {
                        let readable_msg = js_sys::Reflect::get(
                            &error_obj,
                            &wasm_bindgen::JsValue::from_str("readableMsg"),
                        );
                        if let Ok(msg_val) = readable_msg {
                            if let Some(msg_str) = msg_val.as_string() {
                                return Err(RaindexError::CustomError(msg_str));
                            }
                        }
                    }

                    // Fallback to generic error message
                    return Err(RaindexError::CustomError(
                        "Database query failed".to_string(),
                    ));
                }
            }

            // Check for value property
            let value_prop = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("value"));
            if let Ok(value) = value_prop {
                if let Some(json_string) = value.as_string() {
                    final_result = Some(json_string);
                }
            }
        } else if let Some(json_string) = result.as_string() {
            // Fallback for direct JSON string responses
            final_result = Some(json_string);
        }
    }

    if let Some(json_string) = final_result {
        serde_json::from_str(&json_string)
            .map_err(|e| RaindexError::CustomError(format!("JSON deserialization error: {:?}", e)))
    } else {
        Err(RaindexError::CustomError(
            "No valid response from database queries".to_string(),
        ))
    }
}

/// Execute a query that doesn't return data (like CREATE, DELETE, etc.)
pub async fn execute_query_no_result(
    callback: &js_sys::Function,
    sql: &str,
) -> Result<(), RaindexError> {
    let statements = split_sql_statements(sql);

    for statement in statements {
        let result = callback
            .call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(&statement),
            )
            .map_err(|e| RaindexError::CustomError(format!("Callback error: {:?}", e)))?;

        let promise = js_sys::Promise::resolve(&result);
        let future = wasm_bindgen_futures::JsFuture::from(promise);

        let result = future
            .await
            .map_err(|e| RaindexError::CustomError(format!("Promise error: {:?}", e)))?;

        // Check if the result contains an error
        if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
            let error_prop = js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("error"));
            if let Ok(error_val) = error_prop {
                if !error_val.is_undefined() {
                    // Try to get the readableMsg from the error object
                    if let Ok(error_obj) = error_val.dyn_into::<js_sys::Object>() {
                        let readable_msg = js_sys::Reflect::get(
                            &error_obj,
                            &wasm_bindgen::JsValue::from_str("readableMsg"),
                        );
                        if let Ok(msg_val) = readable_msg {
                            if let Some(msg_str) = msg_val.as_string() {
                                return Err(RaindexError::CustomError(msg_str));
                            }
                        }
                    }

                    // Fallback to generic error message
                    return Err(RaindexError::CustomError(
                        "Database query failed".to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[wasm_export]
impl RaindexClient {
    /// Create all database tables
    #[wasm_export(
        js_name = "createTables",
        return_description = "Result of table creation",
        unchecked_return_type = "void"
    )]
    pub async fn create_tables(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
    ) -> Result<(), RaindexError> {
        let sql = get_create_tables_query();
        execute_query_no_result(&callback, &sql).await
    }

    /// Fetch all table names from the database
    #[wasm_export(
        js_name = "fetchAllTables",
        return_description = "JSON array of table names",
        unchecked_return_type = "Array<{name: string}>"
    )]
    pub async fn fetch_all_tables(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
    ) -> Result<Vec<TableResponse>, RaindexError> {
        let sql = get_fetch_all_tables_query();
        execute_query_with_callback::<Vec<TableResponse>>(&callback, &sql).await
    }

    /// Get all active orders
    #[wasm_export(
        js_name = "getActiveOrders",
        return_description = "JSON array of active orders",
        unchecked_return_type = "Array<{order_hash: string, owner: string, creation_time: number, inputs: string, outputs: string, trade_count: number}>"
    )]
    pub async fn get_active_orders(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
    ) -> Result<Vec<ActiveOrderResponse>, RaindexError> {
        let sql = get_active_orders_query();
        execute_query_with_callback(&callback, &sql).await
    }

    /// Get trades for a specific order
    #[wasm_export(
        js_name = "getOrderTrades",
        return_description = "JSON array of trades",
        unchecked_return_type = "Array<{trade_date: string, transaction_hash: string, input_amount: string, output_amount: string}>"
    )]
    pub async fn get_order_trades(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
        #[wasm_export(
            js_name = "orderHash",
            param_description = "The order hash to query trades for"
        )]
        order_hash: String,
    ) -> Result<Vec<OrderTradeResponse>, RaindexError> {
        let sql = get_order_trades_query(&order_hash);
        execute_query_with_callback(&callback, &sql).await
    }

    /// Get vault volumes for a specific order
    #[wasm_export(
        js_name = "getOrderVaultVolumes",
        return_description = "JSON array of vault volumes",
        unchecked_return_type = "Array<{vault_id: string, token: string, decimals: number, total_in: number, total_out: number, net_volume: number, total_volume: number}>"
    )]
    pub async fn get_order_vault_volumes(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
        #[wasm_export(
            js_name = "orderHash",
            param_description = "The order hash to query volumes for"
        )]
        order_hash: String,
    ) -> Result<Vec<OrderVaultVolumeResponse>, RaindexError> {
        let sql = get_order_vaults_volume_query(&order_hash);
        execute_query_with_callback(&callback, &sql).await
    }

    /// Get balance history for a specific vault and token
    #[wasm_export(
        js_name = "getVaultBalanceHistory",
        return_description = "JSON array of vault balance history",
        unchecked_return_type = "Array<{date: string | null, tx_hash: string, balance_change: string, balance_change_type: string, block_number: number, log_index: number}>"
    )]
    pub async fn get_vault_balance_history(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
        #[wasm_export(
            js_name = "vaultId",
            param_description = "The vault ID to query balance history for"
        )]
        vault_id: String,
        #[wasm_export(
            js_name = "token",
            param_description = "The token address to query balance history for"
        )]
        token: String,
    ) -> Result<Vec<VaultBalanceHistoryResponse>, RaindexError> {
        let sql = get_vault_balance_history_query(&vault_id, &token);
        execute_query_with_callback(&callback, &sql).await
    }

    /// Get all vaults with their balances and order associations
    #[wasm_export(
        js_name = "getAllVaults",
        return_description = "JSON array of all vaults",
        unchecked_return_type = "Array<{vault_id: string, token: string, decimals: number, owner: string, balance: number, input_order_hashes: string, output_order_hashes: string}>"
    )]
    pub async fn get_all_vaults(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
    ) -> Result<Vec<AllVaultsResponse>, RaindexError> {
        let sql = get_all_vaults_query();
        execute_query_with_callback(&callback, &sql).await
    }

    /// Clear all database tables
    #[wasm_export(
        js_name = "clearTables",
        return_description = "Result of clearing tables",
        unchecked_return_type = "void"
    )]
    pub async fn clear_tables(
        #[wasm_export(param_description = "JavaScript function to execute SQL queries")]
        callback: js_sys::Function,
    ) -> Result<(), RaindexError> {
        let sql = get_clear_tables_query();
        execute_query_no_result(&callback, &sql).await
    }
}
