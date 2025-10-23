use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVaultBalanceChange {
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: String,
    #[serde(alias = "log_index")]
    pub log_index: u64,
    #[serde(alias = "block_number")]
    pub block_number: u64,
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: u64,
    pub owner: String,
    #[serde(alias = "change_type")]
    pub change_type: String,
    pub token: String,
    #[serde(alias = "vault_id")]
    pub vault_id: String,
    pub delta: String,
    #[serde(alias = "running_balance")]
    pub running_balance: String,
}

pub fn build_fetch_balance_changes_query(vault_id: &str, token: &str) -> String {
    let sanitize_literal = |value: &str| value.replace('\'', "''");
    let vault_id = sanitize_literal(&vault_id.trim().to_lowercase());
    let token = sanitize_literal(&token.trim().to_lowercase());

    QUERY_TEMPLATE
        .replace("'?vault_id'", &format!("'{}'", vault_id))
        .replace("'?token'", &format!("'{}'", token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_sanitized_values() {
        let q = build_fetch_balance_changes_query("  V01'  ", "  0xTo'ken  ");

        // No placeholders remain
        assert!(!q.contains("'?vault_id'"));
        assert!(!q.contains("'?token'"));

        // Lowercased + sanitized literal replacements
        assert!(!q.contains("'?vault_id' AS vault_id"));
        assert!(!q.contains("'?token' AS token"));
        assert!(q.contains("'v01'''"));
        assert!(q.contains("'0xto''ken'"));
    }
}
