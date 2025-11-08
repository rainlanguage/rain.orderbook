use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn upsert_target_watermark_stmt(
    ob_id: &OrderbookIdentifier,
    last_block: u64,
    last_hash: Option<&str>,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::from(ob_id.chain_id as u64),
            SqlValue::from(ob_id.orderbook_address.to_string()),
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
    use alloy::primitives::Address;

    #[test]
    fn upsert_stmt_binds_all_params() {
        let stmt = upsert_target_watermark_stmt(
            &OrderbookIdentifier::new(10, Address::from([0xabu8; 20])),
            123,
            Some("0xhash"),
        );
        assert!(stmt.sql().to_lowercase().contains("on conflict"));
        assert_eq!(stmt.params().len(), 4);
    }

    #[test]
    fn upsert_stmt_sql_matches_template_and_columns() {
        let stmt =
            upsert_target_watermark_stmt(&OrderbookIdentifier::new(1, Address::ZERO), 0, None);
        assert_eq!(stmt.sql(), QUERY_TEMPLATE);
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("insert into target_watermarks"));
        assert!(lower.contains("(chain_id, orderbook_address, last_block, last_hash)"));
        assert!(lower.contains("values (?1, ?2, ?3, ?4)"));
        assert!(lower.contains("on conflict(chain_id, orderbook_address)"));
    }

    #[test]
    fn upsert_stmt_param_order_and_values_with_hash() {
        let chain_id = 100;
        let orderbook = Address::from([0x11u8; 20]);
        let last_block = 42u64;
        let last_hash = "0xdeadbeef";
        let stmt = upsert_target_watermark_stmt(
            &OrderbookIdentifier::new(chain_id, orderbook),
            last_block,
            Some(last_hash),
        );

        let params = stmt.params();
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], SqlValue::U64(chain_id as u64));
        assert_eq!(params[1], SqlValue::Text(orderbook.to_string()));
        assert_eq!(params[2], SqlValue::U64(last_block));
        assert_eq!(params[3], SqlValue::Text(last_hash.to_string()));
    }

    #[test]
    fn upsert_stmt_param_order_and_values_without_hash_null() {
        let chain_id = 5;
        let orderbook = Address::from([0xabu8; 20]);
        let last_block = 7u64;
        let stmt = upsert_target_watermark_stmt(
            &OrderbookIdentifier::new(chain_id, orderbook),
            last_block,
            None,
        );

        let params = stmt.params();
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], SqlValue::U64(chain_id as u64));
        assert_eq!(params[1], SqlValue::Text(orderbook.to_string()));
        assert_eq!(params[2], SqlValue::U64(last_block));
        assert_eq!(params[3], SqlValue::Null);
    }
}
