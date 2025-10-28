use crate::local_db::query::{SqlStatement, SqlValue};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn upsert_target_watermark_stmt(
    chain_id: u64,
    orderbook_address: &str,
    last_block: u64,
    last_hash: Option<&str>,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::from(chain_id),
            SqlValue::from(orderbook_address.to_string()),
            SqlValue::from(last_block),
            match last_hash {
                Some(h) => SqlValue::from(h.to_string()),
                None => SqlValue::Null,
            },
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_stmt_binds_all_params() {
        let stmt = upsert_target_watermark_stmt(10, "0xabc", 123, Some("0xhash"));
        assert!(stmt.sql().to_lowercase().contains("on conflict"));
        assert_eq!(stmt.params().len(), 4);
    }
}
