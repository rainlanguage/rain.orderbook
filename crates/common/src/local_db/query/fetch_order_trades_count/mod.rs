use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbTradeCountRow {
    #[serde(alias = "trade_count")]
    pub trade_count: u64,
}

pub fn build_fetch_trade_count_query(
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
        .replace("?filter_start_timestamp", &filter_start_timestamp)
        .replace("?filter_end_timestamp", &filter_end_timestamp)
}

pub fn extract_trade_count(rows: &[LocalDbTradeCountRow]) -> u64 {
    rows.first().map(|row| row.trade_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_time_filters_and_sanitization() {
        let q = build_fetch_trade_count_query(" 0xABC'DEF ", Some(1000), Some(2000));

        // Placeholders should be gone
        assert!(!q.contains("?filter_start_timestamp"));
        assert!(!q.contains("?filter_end_timestamp"));
        assert!(!q.contains("?order_hash"));

        // Time filters present
        assert!(q.contains("block_timestamp >= 1000"));
        assert!(q.contains("block_timestamp <= 2000"));

        // Order hash trimmed, lowercased, and quotes sanitized
        // We expect lower('0xabc''def') to appear at least once.
        assert!(q.contains("lower('0xabc''def')"));
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let q = build_fetch_trade_count_query("hash", None, None);
        assert!(!q.contains("block_timestamp >="));
        assert!(!q.contains("block_timestamp <="));
        assert!(!q.contains("?filter_start_timestamp"));
        assert!(!q.contains("?filter_end_timestamp"));
    }

    #[test]
    fn extract_trade_count_behaviour() {
        let rows = vec![LocalDbTradeCountRow { trade_count: 7 }];
        assert_eq!(extract_trade_count(&rows), 7);
        let none: Vec<LocalDbTradeCountRow> = vec![];
        assert_eq!(extract_trade_count(&none), 0);
    }
}
