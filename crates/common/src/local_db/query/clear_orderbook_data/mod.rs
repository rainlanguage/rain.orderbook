use crate::local_db::query::{SqlStatement, SqlValue};
use alloy::primitives::Address;

pub const CLEAR_ORDERBOOK_DATA_SQL: &str = include_str!("query.sql");

pub fn clear_orderbook_data_stmt(chain_id: u64, orderbook_address: Address) -> SqlStatement {
    SqlStatement::new_with_params(
        CLEAR_ORDERBOOK_DATA_SQL,
        [
            SqlValue::from(chain_id),
            SqlValue::from(orderbook_address.to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use std::collections::HashSet;

    const TABLES_EXCLUDED_FROM_CLEAR: &[&str] = &["db_metadata"];

    fn normalize_ident(s: &str) -> String {
        let s = s.trim();
        let s = s.trim_matches('`').trim_matches('"');
        let s = if s.starts_with('[') && s.ends_with(']') && s.len() >= 2 {
            &s[1..s.len() - 1]
        } else {
            s
        };
        s.to_lowercase()
    }

    #[test]
    fn stmt_binds_expected_params() {
        let addr = Address::from([0x11; 20]);
        let stmt = clear_orderbook_data_stmt(137, addr);

        assert_eq!(stmt.sql(), CLEAR_ORDERBOOK_DATA_SQL);
        assert_eq!(
            stmt.params(),
            &[SqlValue::U64(137), SqlValue::Text(addr.to_string()),]
        );
    }

    #[test]
    fn sql_covers_expected_tables() {
        let sql = CLEAR_ORDERBOOK_DATA_SQL.to_ascii_lowercase();
        let expected_tables: HashSet<String> = REQUIRED_TABLES
            .iter()
            .copied()
            .filter(|table| !TABLES_EXCLUDED_FROM_CLEAR.contains(table))
            .map(normalize_ident)
            .collect();

        let mut deleted_tables = HashSet::new();
        let mut tables_missing_scope = Vec::new();
        let needle = "delete from ";
        let mut offset = 0usize;

        while let Some(idx) = sql[offset..].find(needle) {
            let start = offset + idx + needle.len();
            let rest = &sql[start..];
            let end = rest.find('\n').unwrap_or(rest.len());
            let table_name = normalize_ident(rest[..end].trim());
            if !table_name.is_empty() {
                let stmt_start = offset + idx;
                let stmt_rest = &sql[stmt_start..];
                let stmt_end = stmt_rest.find(';').unwrap_or_else(|| {
                    panic!("DELETE statement for table {table_name} missing ';'")
                });
                let stmt = &stmt_rest[..stmt_end];
                let has_chain_guard = stmt.contains("where chain_id = ?1");
                let has_address_guard = stmt.contains("lower(orderbook_address) = lower(?2)");
                if !(has_chain_guard && has_address_guard) {
                    tables_missing_scope.push(table_name.clone());
                }

                deleted_tables.insert(table_name);
            }
            offset = start + end;
        }

        for table in &expected_tables {
            assert!(
                deleted_tables.contains(table),
                "missing delete for table {table}"
            );
        }

        let unexpected_tables: HashSet<_> = deleted_tables
            .difference(&expected_tables)
            .cloned()
            .collect();
        assert!(
            unexpected_tables.is_empty(),
            "unexpected delete for tables {:?}",
            unexpected_tables
        );

        assert!(
            tables_missing_scope.is_empty(),
            "delete statements missing scope guards for tables {:?}",
            tables_missing_scope
        );

        assert!(
            sql.contains("begin transaction;"),
            "transaction not started"
        );
        assert!(sql.contains("commit;"), "transaction not committed");
    }
}
