use crate::local_db::query::SqlStatement;

pub const CREATE_TABLES_SQL: &str = include_str!("query.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "db_metadata",
    "target_watermarks",
    "sync_status",
    "raw_events",
    "deposits",
    "withdrawals",
    "order_events",
    "order_ios",
    "take_orders",
    "take_order_contexts",
    "context_values",
    "clear_v3_events",
    "after_clear_v2_events",
    "meta_events",
    "erc20_tokens",
    "interpreter_store_sets",
];

pub fn create_tables_sql() -> &'static str {
    CREATE_TABLES_SQL
}

pub fn create_tables_stmt() -> SqlStatement {
    SqlStatement::new(CREATE_TABLES_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
    fn required_tables_match_sql_create_statements() {
        let sql = create_tables_sql();
        let sql_lower = sql.to_lowercase();

        let mut parsed_tables: HashSet<String> = HashSet::new();
        let mut offset = 0usize;
        let needle = "create table";

        while let Some(rel_idx) = sql_lower[offset..].find(needle) {
            let start = offset + rel_idx;
            // Find the opening parenthesis that starts the column list
            let open_paren_rel = sql_lower[start..]
                .find('(')
                .expect("CREATE TABLE without opening '('");
            let open_paren = start + open_paren_rel;

            // Slice between 'create table' and '('
            let name_part = &sql[start + needle.len()..open_paren];
            // Tokenize and take the last token which should be the table name,
            // handling the optional 'IF NOT EXISTS'
            let tokens: Vec<&str> = name_part.split_whitespace().collect();
            if let Some(last) = tokens.last() {
                let table = normalize_ident(last);
                // Exclude accidental matches on CREATE TABLE that aren't actual table definitions
                if !table.is_empty() {
                    parsed_tables.insert(table);
                }
            }

            // Advance after this '('
            offset = open_paren + 1;
        }

        let required_tables: HashSet<String> =
            REQUIRED_TABLES.iter().map(|t| normalize_ident(t)).collect();

        // Compute diffs
        let missing: HashSet<_> = required_tables
            .difference(&parsed_tables)
            .cloned()
            .collect();
        let extra: HashSet<_> = parsed_tables
            .difference(&required_tables)
            .cloned()
            .collect();

        assert!(
            missing.is_empty() && extra.is_empty(),
            "Table set mismatch. Missing in SQL: {:?}; Extra in SQL: {:?}",
            missing,
            extra
        );
    }
}
