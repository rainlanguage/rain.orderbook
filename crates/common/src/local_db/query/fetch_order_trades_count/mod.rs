use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbTradeCountRow {
    #[serde(alias = "trade_count")]
    pub trade_count: u64,
}

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

pub fn build_fetch_trade_count_stmt(
    order_hash: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    // ?1: order hash
    stmt.push(SqlValue::Text(order_hash.to_string()));
    // Optional time filters
    if let (Some(start), Some(end)) = (start_timestamp, end_timestamp) {
        if start > end {
            return Err(SqlBuildError::new("start_timestamp > end_timestamp"));
        }
    }

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

pub fn extract_trade_count(rows: &[LocalDbTradeCountRow]) -> u64 {
    rows.first().map(|row| row.trade_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_time_filters() {
        let stmt = build_fetch_trade_count_stmt("0xABC'DEF", Some(1000), Some(2000)).unwrap();
        // Time filter clauses present
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
        // Params include order hash and two timestamps
        assert_eq!(stmt.params.len(), 3);
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let stmt = build_fetch_trade_count_stmt("hash", None, None).unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 1);
    }

    #[test]
    fn extract_trade_count_behaviour() {
        let rows = vec![LocalDbTradeCountRow { trade_count: 7 }];
        assert_eq!(extract_trade_count(&rows), 7);
        let none: Vec<LocalDbTradeCountRow> = vec![];
        assert_eq!(extract_trade_count(&none), 0);
    }
}
