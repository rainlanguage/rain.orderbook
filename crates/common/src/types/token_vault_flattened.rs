use crate::csv::TryIntoCsv;
use rain_orderbook_subgraph_client::types::vaults_list::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: Bytes,
    pub vault_id: BigInt,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: i32,
    pub token_address: String,
    pub balance_display: BigInt,
    pub balance: BigInt,
}

impl From<Vault> for TokenVaultFlattened {
    fn from(val: Vault) -> Self {
        Self {
            id: val.id.0,
            owner: val.owner,
            vault_id: val.vault_id,
            token_name: val.token.name,
            token_symbol: val.token.symbol,
            token_decimals: val.token.decimals,
            token_address: val.token.id.into_inner(),
            balance_display: val.balance_display,
            balance: val.balance,
        }
    }
}

impl TryIntoCsv<TokenVaultFlattened> for Vec<TokenVaultFlattened> {}
