use crate::csv::TryIntoCsv;
use rain_orderbook_subgraph_client::types::vaults_list;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: vaults_list::Bytes,
    pub vault_id: vaults_list::BigInt,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: i32,
    pub token_address: String,
    pub balance_display: vaults_list::BigDecimal,
    pub balance: vaults_list::BigInt,
}

impl From<vaults_list::TokenVault> for TokenVaultFlattened {
    fn from(val: vaults_list::TokenVault) -> Self {
        Self {
            id: val.id.into_inner(),
            owner: val.owner.id,
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
