use serde::{Deserialize, Serialize};

pub const FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Erc20TokenRow {
    pub chain_id: u32,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// Builds the SQL statement used to load ERC20 metadata for the supplied
/// addresses. Returns `None` when the address list is empty, allowing callers
/// to short-circuit database work.
pub fn build_fetch_query(chain_id: u32, addresses: &[String]) -> Option<String> {
    if addresses.is_empty() {
        return None;
    }

    let in_clause = addresses
        .iter()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL
        .replace("?chain_id", &chain_id.to_string())
        .replace("?addresses_in", &in_clause);

    Some(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_addresses_returns_none() {
        let q = build_fetch_query(1, &[]);
        assert!(q.is_none());
    }

    #[test]
    fn builds_in_clause_and_chain_id_and_sanitizes() {
        let addrs = vec!["0xAbc".to_string(), "O'Malley".to_string()];
        let q = build_fetch_query(137, &addrs).expect("should build query");

        // Placeholders should be gone
        assert!(!q.contains("?chain_id"));
        assert!(!q.contains("?addresses_in"));

        // Chain id inserted
        assert!(q.contains("WHERE chain_id = 137"));

        // Addresses quoted and sanitized
        assert!(q.contains("address IN ('0xAbc', 'O''Malley')"));
    }
}
