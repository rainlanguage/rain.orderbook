use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrderTrade {
    pub trade_kind: String,
    pub orderbook_address: Address,
    pub order_hash: Bytes,
    pub order_owner: Address,
    pub order_nonce: String,
    pub transaction_hash: Bytes,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub transaction_sender: Address,
    pub input_vault_id: U256,
    pub input_token: Address,
    pub input_token_name: Option<String>,
    pub input_token_symbol: Option<String>,
    pub input_token_decimals: Option<u8>,
    pub input_delta: String,
    pub input_running_balance: Option<String>,
    pub output_vault_id: U256,
    pub output_token: Address,
    pub output_token_name: Option<String>,
    pub output_token_symbol: Option<String>,
    pub output_token_decimals: Option<u8>,
    pub output_delta: String,
    pub output_running_balance: Option<String>,
    pub trade_id: String,
}

/// Builds the SQL statement for retrieving order trades within the specified window.
const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";

const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

pub fn build_fetch_order_trades_stmt(
    ob_id: &OrderbookIdentifier,
    order_hash: Bytes,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));
    stmt.push(SqlValue::from(order_hash));

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
    use std::str::FromStr;

    use alloy::primitives::Address;

    use super::*;

    #[test]
    fn builds_with_chain_id_and_filters() {
        let stmt = build_fetch_order_trades_stmt(
            &OrderbookIdentifier::new(137, Address::ZERO),
            Bytes::from_str("0xdeadface").unwrap(),
            Some(11),
            Some(22),
        )
        .unwrap();
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
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text("0xdeadface".to_string()));
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let stmt = build_fetch_order_trades_stmt(
            &OrderbookIdentifier::new(1, Address::ZERO),
            Bytes::from_str("0xdeadbeef").unwrap(),
            None,
            None,
        )
        .unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 3);
        // Order of fixed params: chain id (?1), orderbook (?2), order hash (?3)
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text("0xdeadbeef".to_string()));
    }
}
