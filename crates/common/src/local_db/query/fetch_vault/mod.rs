use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVault {
    #[serde(alias = "vaultId")]
    pub vault_id: String,
    pub token: String,
    pub owner: String,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
    #[serde(alias = "tokenName")]
    pub token_name: String,
    #[serde(alias = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(alias = "tokenDecimals")]
    pub token_decimals: u8,
    pub balance: String,
    #[serde(alias = "inputOrders")]
    pub input_orders: Option<String>,
    #[serde(alias = "outputOrders")]
    pub output_orders: Option<String>,
}

pub fn build_fetch_vault_query(chain_id: u32, vault_id: &str, token: &str) -> String {
    QUERY_TEMPLATE
        .replace("'?vault_id'", &format!("'{}'", vault_id))
        .replace("'?token'", &format!("'{}'", token))
        .replace("'?chain_id'", &format!("'{}'", chain_id))
}

/// Parses the IO annotation string emitted by the database into a sorted list of
/// `(index, vault_id, token)` tuples.
pub fn parse_io_indexed_pairs(io: &Option<String>) -> Vec<(usize, String, String)> {
    let mut items: Vec<(usize, String, String)> = vec![];
    if let Some(s) = io {
        for part in s.split(',') {
            let mut segs = part.split(':');
            let idx = segs.next().map(|x| x.trim());
            let vault_id = segs.next().map(|x| x.trim());
            let token = segs.next().map(|x| x.trim());
            if let (Some(idx), Some(vault_id), Some(token)) = (idx, vault_id, token) {
                if let Ok(index) = idx.parse::<usize>() {
                    items.push((index, vault_id.to_string(), token.to_string()));
                }
            }
        }
        items.sort_by_key(|(i, _, _)| *i);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_query_replacing_placeholders() {
        let q = build_fetch_vault_query(10, "0x01", "0xabc");
        assert!(!q.contains("'?vault_id'"));
        assert!(!q.contains("'?token'"));
        assert!(!q.contains("'?chain_id'"));
        assert!(q.contains("et.chain_id = '10'"));
        assert!(!q.contains("'?vault_id' AS vault_id"));
        assert!(!q.contains("'?token' AS token"));
        assert!(q.contains("'0x01'"));
        assert!(q.contains("'0xabc'"));
    }

    #[test]
    fn parse_io_pairs_none_and_empty() {
        let none: Option<String> = None;
        assert!(parse_io_indexed_pairs(&none).is_empty());

        let some = Some(String::new());
        assert!(parse_io_indexed_pairs(&some).is_empty());
    }

    #[test]
    fn parse_io_pairs_valid_and_sorted() {
        let s = Some("3:v3:t3,1:v1:t1,2:v2:t2".to_string());
        let got = parse_io_indexed_pairs(&s);
        assert_eq!(
            got,
            vec![
                (1, "v1".to_string(), "t1".to_string()),
                (2, "v2".to_string(), "t2".to_string()),
                (3, "v3".to_string(), "t3".to_string()),
            ]
        );
    }

    #[test]
    fn parse_io_pairs_ignores_invalid_segments() {
        // bad index, missing fields, and one valid
        let s = Some("x:a:b, 7:only_two, 5:ok:tok".to_string());
        let got = parse_io_indexed_pairs(&s);
        assert_eq!(got, vec![(5, "ok".to_string(), "tok".to_string())]);
    }
}
