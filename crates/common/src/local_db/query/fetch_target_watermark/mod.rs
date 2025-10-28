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
    use super::*;

    #[test]
    fn fetch_stmt_binds_params() {
        let stmt = fetch_target_watermark_stmt(10, Address::ZERO);
        assert!(stmt.sql().to_lowercase().contains("from target_watermarks"));
        assert_eq!(stmt.params().len(), 2);
    }
}
