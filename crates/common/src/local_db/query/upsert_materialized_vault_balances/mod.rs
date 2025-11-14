use crate::local_db::{
    query::{SqlStatement, SqlValue},
    OrderbookIdentifier,
};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn upsert_materialized_vault_balances_stmt(
    ob_id: &OrderbookIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::from(ob_id.chain_id as u64),
            SqlValue::from(ob_id.orderbook_address.to_string()),
            SqlValue::from(start_block),
            SqlValue::from(end_block),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn stmt_binds_all_params() {
        let ob_id = OrderbookIdentifier::new(111, Address::from([0x11u8; 20]));
        let stmt = upsert_materialized_vault_balances_stmt(&ob_id, 100, 200);
        assert_eq!(stmt.sql(), QUERY_TEMPLATE);
        assert_eq!(stmt.params().len(), 4);
        assert_eq!(stmt.params()[0], SqlValue::U64(111));
        assert_eq!(
            stmt.params()[1],
            SqlValue::Text(ob_id.orderbook_address.to_string())
        );
        assert_eq!(stmt.params()[2], SqlValue::U64(100));
        assert_eq!(stmt.params()[3], SqlValue::U64(200));
    }

    #[test]
    fn stmt_targets_materialized_table() {
        let stmt = upsert_materialized_vault_balances_stmt(
            &OrderbookIdentifier::new(1, Address::ZERO),
            0,
            10,
        );
        let sql_lower = stmt.sql().to_lowercase();
        assert!(sql_lower.contains("insert into materialized_vault_balances"));
        assert!(
            sql_lower.contains("on conflict (chain_id, orderbook_address, owner, token, vault_id)")
        );
    }

    #[test]
    fn stmt_filters_block_range() {
        let stmt = upsert_materialized_vault_balances_stmt(
            &OrderbookIdentifier::new(3, Address::from([0x55; 20])),
            123,
            456,
        );
        let sql = stmt.sql();
        assert!(
            sql.contains("vd.block_number BETWEEN ?3 AND ?4"),
            "missing block filter"
        );
    }

    #[test]
    fn stmt_skips_zero_balance_batches() {
        let stmt = upsert_materialized_vault_balances_stmt(
            &OrderbookIdentifier::new(4, Address::ZERO),
            0,
            0,
        );
        let sql = stmt.sql().to_lowercase();
        assert!(
            sql.contains("having not float_is_zero"),
            "missing HAVING NOT FLOAT_IS_ZERO"
        );
    }

    #[test]
    fn stmt_updates_conflict_columns() {
        let stmt = upsert_materialized_vault_balances_stmt(
            &OrderbookIdentifier::new(5, Address::ZERO),
            0,
            1,
        );
        let sql = stmt.sql().to_lowercase();
        assert!(
            sql.contains("balance = float_add"),
            "missing FLOAT_ADD in conflict update"
        );
        assert!(sql.contains("last_block = case"), "missing last_block case");
        assert!(
            sql.contains("last_log_index = case"),
            "missing last_log_index case"
        );
    }
}
