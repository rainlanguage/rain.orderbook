use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;
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

/// Builds the SQL statement for retrieving order trades within the specified window.
const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";

const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

pub fn build_fetch_order_trades_stmt(
    chain_id: u32,
    orderbook_address: Address,
    order_hash: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    // ?1: chain id, ?2: orderbook address, ?3: order hash
    stmt.push(SqlValue::I64(chain_id as i64));
    stmt.push(SqlValue::Text(orderbook_address.to_string()));
    stmt.push(SqlValue::Text(order_hash.trim().to_string()));

    // Optional time filters
    let start_param = if let Some(v) = start_timestamp {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!(
                "start_timestamp out of range for i64: {} ({})",
                v, e
            ))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(START_TS_CLAUSE, START_TS_BODY, start_param)?;

    let end_param = if let Some(v) = end_timestamp {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(END_TS_CLAUSE, END_TS_BODY, end_param)?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_chain_id_and_filters() {
        let stmt =
            build_fetch_order_trades_stmt(137, Address::ZERO, "AbC'X", Some(11), Some(22)).unwrap();
        // Fixed params
        assert!(stmt.sql.contains("et_in.chain_id = ?1"));
        assert!(stmt.sql.contains("et_out.chain_id = ?1"));
        // Dynamic param clauses inserted
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
        // First three fixed params: chain id (?1), orderbook address (?2), order hash (?3)
        assert_eq!(stmt.params.len(), 5); // includes start and end
        assert_eq!(stmt.params[0], SqlValue::I64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text("AbC'X".to_string()));
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let stmt = build_fetch_order_trades_stmt(1, Address::ZERO, "hash", None, None).unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 3);
        // Order of fixed params: chain id (?1), orderbook (?2), order hash (?3)
        assert_eq!(stmt.params[0], SqlValue::I64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text("hash".to_string()));
    }
}
