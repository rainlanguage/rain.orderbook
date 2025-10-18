use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrderTrade {
    #[serde(alias = "trade_kind")]
    pub trade_kind: String,
    #[serde(alias = "orderbook_address")]
    pub orderbook_address: String,
    #[serde(alias = "order_hash")]
    pub order_hash: String,
    #[serde(alias = "order_owner")]
    pub order_owner: String,
    #[serde(alias = "order_nonce")]
    pub order_nonce: String,
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "log_index")]
    pub log_index: u64,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "transaction_sender")]
    pub transaction_sender: String,
    #[serde(alias = "input_vault_id")]
    pub input_vault_id: String,
    #[serde(alias = "input_token")]
    pub input_token: String,
    #[serde(alias = "input_token_name")]
    pub input_token_name: Option<String>,
    #[serde(alias = "input_token_symbol")]
    pub input_token_symbol: Option<String>,
    #[serde(alias = "input_token_decimals")]
    pub input_token_decimals: Option<u8>,
    #[serde(alias = "input_delta")]
    pub input_delta: String,
    #[serde(alias = "input_running_balance")]
    pub input_running_balance: Option<String>,
    #[serde(alias = "output_vault_id")]
    pub output_vault_id: String,
    #[serde(alias = "output_token")]
    pub output_token: String,
    #[serde(alias = "output_token_name")]
    pub output_token_name: Option<String>,
    #[serde(alias = "output_token_symbol")]
    pub output_token_symbol: Option<String>,
    #[serde(alias = "output_token_decimals")]
    pub output_token_decimals: Option<u8>,
    #[serde(alias = "output_delta")]
    pub output_delta: String,
    #[serde(alias = "output_running_balance")]
    pub output_running_balance: Option<String>,
    #[serde(alias = "trade_id")]
    pub trade_id: String,
}

/// Builds the SQL query for retrieving order trades within the specified window.
pub fn build_fetch_order_trades_query(
    chain_id: u32,
    order_hash: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> String {
    let sanitize_literal = |value: &str| value.replace('\'', "''");

    let order_hash = sanitize_literal(&order_hash.trim().to_lowercase());

    let filter_start_timestamp = start_timestamp
        .map(|ts| format!("\nAND block_timestamp >= {}\n", ts))
        .unwrap_or_default();
    let filter_end_timestamp = end_timestamp
        .map(|ts| format!("\nAND block_timestamp <= {}\n", ts))
        .unwrap_or_default();

    QUERY_TEMPLATE
        .replace("'?order_hash'", &format!("'{}'", order_hash))
        .replace("'?chain_id'", &chain_id.to_string())
        .replace("?filter_start_timestamp", &filter_start_timestamp)
        .replace("?filter_end_timestamp", &filter_end_timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_chain_id_sanitized_order_hash_and_filters() {
        let q = build_fetch_order_trades_query(137, "  AbC'X  ", Some(11), Some(22));

        // No placeholders remain
        assert!(!q.contains("?filter_start_timestamp"));
        assert!(!q.contains("?filter_end_timestamp"));
        assert!(!q.contains("'?order_hash'"));
        assert!(!q.contains("'?chain_id'"));

        // Chain id appears in erc20 token JOINs
        assert!(q.contains("et_in.chain_id = 137"));
        assert!(q.contains("et_out.chain_id = 137"));

        // Time filters
        assert!(q.contains("block_timestamp >= 11"));
        assert!(q.contains("block_timestamp <= 22"));

        // Sanitized, lowercased order hash
        assert!(q.contains("lower('abc''x')"));
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let q = build_fetch_order_trades_query(1, "hash", None, None);
        assert!(!q.contains("block_timestamp >="));
        assert!(!q.contains("block_timestamp <="));
        assert!(!q.contains("?filter_start_timestamp"));
        assert!(!q.contains("?filter_end_timestamp"));
    }
}
