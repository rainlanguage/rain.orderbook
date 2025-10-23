const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn build_update_last_synced_block_query(block_number: u64) -> String {
    QUERY_TEMPLATE.replace("?block_number", &block_number.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_update_query() {
        let q = build_update_last_synced_block_query(999);
        assert!(q.contains("last_synced_block = 999"));
        assert!(!q.contains("?block_number"));
    }
}
