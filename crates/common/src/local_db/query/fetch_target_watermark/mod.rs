use crate::local_db::query::{SqlStatement, SqlValue};
use alloy::primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

pub const FETCH_TARGET_WATERMARK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetWatermarkRow {
    pub chain_id: u32,
    pub orderbook_address: Address,
    pub last_block: u64,
    pub last_hash: Option<Bytes>,
    pub updated_at: Option<String>,
}

pub fn fetch_target_watermark_stmt(chain_id: u32, orderbook_address: Address) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_TARGET_WATERMARK_SQL,
        [
            SqlValue::from(chain_id as u64),
            SqlValue::from(orderbook_address.to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::address;

    #[test]
    fn fetch_stmt_binds_params() {
        let stmt = fetch_target_watermark_stmt(10, Address::ZERO);
        assert!(stmt.sql().to_lowercase().contains("from target_watermarks"));
        assert_eq!(stmt.params().len(), 2);
    }

    #[test]
    fn fetch_stmt_sql_matches_template_and_where_clause() {
        let stmt = fetch_target_watermark_stmt(1, Address::ZERO);
        // Exact template equality (comes from include_str!)
        assert_eq!(stmt.sql(), FETCH_TARGET_WATERMARK_SQL);
        // Defensive: check placeholders and where clause shape
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("where chain_id = ?1 and orderbook_address = ?2"));
    }

    #[test]
    fn fetch_stmt_param_order_and_values() {
        let chain_id = 111u32;
        let addr = Address::repeat_byte(0xab);
        let stmt = fetch_target_watermark_stmt(chain_id, addr);

        let params = stmt.params();
        assert_eq!(params.len(), 2);
        assert_eq!(
            params[0],
            SqlValue::U64(chain_id as u64),
            "first param must be chain_id as U64"
        );
        assert_eq!(
            params[1],
            SqlValue::Text(addr.to_string()),
            "second param must be orderbook_address as hex string"
        );
        // Ensure address string formatting is 0x-prefixed lowercase hex
        let SqlValue::Text(s) = &params[1] else {
            panic!("expected text param")
        };
        assert!(s.starts_with("0x"));
        assert_eq!(s.len(), 42); // 0x + 40 hex chars
        assert_eq!(s, &addr.to_string());
    }

    #[test]
    fn target_watermark_row_serde_roundtrip_none_fields() {
        let row = TargetWatermarkRow {
            chain_id: 5,
            orderbook_address: Address::ZERO,
            last_block: 123,
            last_hash: None,
            updated_at: None,
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: TargetWatermarkRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }

    #[test]
    fn target_watermark_row_serde_roundtrip_some_fields() {
        let row = TargetWatermarkRow {
            chain_id: 10,
            orderbook_address: address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            last_block: 999_999,
            last_hash: Some(Bytes::from_str("0xdeadbeef").unwrap()),
            updated_at: Some("2024-01-01T00:00:00Z".to_owned()),
        };
        let j = serde_json::to_value(&row).expect("serialize");
        let rt: TargetWatermarkRow = serde_json::from_value(j).expect("deserialize");
        assert_eq!(rt, row);
    }
}
