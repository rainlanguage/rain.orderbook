use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]

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
    #[serde(alias = "inputOrderHashes")]
    pub input_order_hashes: Option<String>,
    #[serde(alias = "outputOrderHashes")]
    pub output_order_hashes: Option<String>,
}

impl LocalDbQuery {
    pub async fn fetch_vault(
        db_callback: &js_sys::Function,
        chain_id: u32,
        vault_id: &str,
        token: &str,
    ) -> Result<Option<LocalDbVault>, LocalDbQueryError> {
        let sql = QUERY
            .replace("'?vault_id'", &format!("'{}'", vault_id))
            .replace("'?token'", &format!("'{}'", token))
            .replace("'?chain_id'", &format!("'{}'", chain_id));

        let rows = LocalDbQuery::execute_query_json::<Vec<LocalDbVault>>(db_callback, &sql).await?;
        Ok(rows.into_iter().next())
    }

    // Parse a comma-separated IO string like "0:vault:token,1:vault:token" into sorted triples
    pub fn parse_io_indexed_pairs(io: &Option<String>) -> Vec<(usize, String, String)> {
        let mut items: Vec<(usize, String, String)> = vec![];
        if let Some(s) = io {
            for part in s.split(',') {
                let mut segs = part.split(':');
                if let (Some(idx), Some(vault_id), Some(token)) =
                    (segs.next(), segs.next(), segs.next())
                {
                    if let Ok(index) = idx.parse::<usize>() {
                        items.push((index, vault_id.to_string(), token.to_string()));
                    }
                }
            }
            items.sort_by_key(|(i, _, _)| *i);
        }
        items
    }

    // Given an IO string, fetch corresponding vaults in order
    pub async fn fetch_vaults_for_io_string(
        db_callback: &js_sys::Function,
        chain_id: u32,
        io: &Option<String>,
    ) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
        let ios = Self::parse_io_indexed_pairs(io);
        let mut vaults = Vec::with_capacity(ios.len());
        for (_, vault_id, token) in ios.iter() {
            if let Some(v) = Self::fetch_vault(db_callback, chain_id, vault_id, token).await? {
                vaults.push(v);
            }
        }
        Ok(vaults)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::{
            create_sql_capturing_callback, create_success_callback,
        };
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_vault_parses_data() {
            let vault = LocalDbVault {
                vault_id: "0x01".into(),
                token: "0xaaa".into(),
                owner: "0x1111111111111111111111111111111111111111".into(),
                orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                token_name: "Token A".into(),
                token_symbol: "TA".into(),
                token_decimals: 6,
                balance: "0x10".into(),
                input_order_hashes: Some(
                    "0xabc0000000000000000000000000000000000000000000000000000000000001".into(),
                ),
                output_order_hashes: None,
            };
            let json_data = serde_json::to_string(&vec![vault.clone()]).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_vault(&callback, 1, "0x01", "0xaaa").await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert!(data.is_some());
            let data = data.unwrap();
            assert_eq!(data.vault_id, vault.vault_id);
            assert_eq!(data.token, vault.token);
            assert_eq!(data.owner, vault.owner);
            assert_eq!(data.orderbook_address, vault.orderbook_address);
            assert_eq!(data.token_name, vault.token_name);
            assert_eq!(data.token_symbol, vault.token_symbol);
            assert_eq!(data.token_decimals, vault.token_decimals);
            assert_eq!(data.balance, vault.balance);
            assert_eq!(data.input_order_hashes, vault.input_order_hashes);
            assert_eq!(data.output_order_hashes, vault.output_order_hashes);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vault_replaces_placeholders() {
            let captured = Rc::new(RefCell::new(String::new()));
            // Provide empty result array
            let callback = create_sql_capturing_callback("[]", captured.clone());

            let _ = LocalDbQuery::fetch_vault(&callback, 137, "0xdead", "0xbeef").await;

            let sql = captured.borrow();
            assert!(sql.contains("'0xdead'"));
            assert!(sql.contains("'0xbeef'"));
            assert!(sql.contains("137"));
            assert!(!sql.contains("?vault_id"));
            assert!(!sql.contains("?token"));
            assert!(!sql.contains("?chain_id"));
        }

        #[wasm_bindgen_test]
        async fn test_parse_io_indexed_pairs() {
            let io = Some("2:0x02:0xbbb,0:0x00:0xaaa,1:0x01:0xaaa".into());
            let parsed = LocalDbQuery::parse_io_indexed_pairs(&io);
            assert_eq!(parsed.len(), 3);
            assert_eq!(parsed[0].0, 0);
            assert_eq!(parsed[0].1, "0x00");
            assert_eq!(parsed[1].0, 1);
            assert_eq!(parsed[1].1, "0x01");
            assert_eq!(parsed[2].0, 2);
            assert_eq!(parsed[2].1, "0x02");
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_for_io_string_collects_multiple() {
            let sample = LocalDbVault {
                vault_id: "0xV".into(),
                token: "0xT".into(),
                owner: "0xOwner".into(),
                orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                token_name: "Token X".into(),
                token_symbol: "TX".into(),
                token_decimals: 18,
                balance: "0x10".into(),
                input_order_hashes: Some("0xabc".into()),
                output_order_hashes: None,
            };
            let json_data = serde_json::to_string(&vec![sample.clone()]).unwrap();
            let callback =
                crate::raindex_client::local_db::query::tests::create_success_callback(&json_data);

            let io = Some("1:0x01:0xaaa,0:0x02:0xbbb".into());
            let result = LocalDbQuery::fetch_vaults_for_io_string(&callback, 1, &io).await;
            assert!(result.is_ok());
            let list = result.unwrap();
            assert_eq!(list.len(), 2);
            // both entries equal the sample because the callback returns the same row
            assert_eq!(list[0].orderbook_address, sample.orderbook_address);
            assert_eq!(list[1].balance, sample.balance);
        }
    }
}
