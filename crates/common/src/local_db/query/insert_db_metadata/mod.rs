use crate::local_db::query::{SqlStatement, SqlValue};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn insert_db_metadata_stmt(version: u32) -> SqlStatement {
    SqlStatement::new_with_params(QUERY_TEMPLATE, [SqlValue::from(version as u64)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_stmt_binds_version() {
        let stmt = insert_db_metadata_stmt(7);
        assert!(stmt
            .sql()
            .to_lowercase()
            .starts_with("insert into db_metadata"));
        assert_eq!(stmt.params().len(), 1);
    }
}
