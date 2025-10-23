use crate::local_db::query::{SqlStatement, SqlValue};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn build_update_last_synced_block_stmt(block_number: u64) -> SqlStatement {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    // ?1 -> block number
    stmt.push(SqlValue::I64(block_number as i64));
    stmt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_update_query() {
        let stmt = build_update_last_synced_block_stmt(999);
        assert!(stmt.sql.contains("last_synced_block = ?1"));
        assert_eq!(stmt.params.len(), 1);
    }
}
