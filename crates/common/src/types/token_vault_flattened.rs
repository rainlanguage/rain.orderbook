use crate::csv::TryIntoCsv;
use alloy::primitives::{utils::format_units, U256};
use rain_orderbook_subgraph_client::types::vaults_list::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: Bytes,
    pub vault_id: BigInt,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<BigInt>,
    pub token_address: String,
    pub balance_display: String,
    pub balance: BigInt,
}

impl TryFrom<Vault> for TokenVaultFlattened {
    type Error = FlattenError;

    fn try_from(val: Vault) -> Result<Self, Self::Error> {
        let balance_parsed = val.balance.0.parse::<U256>()?;
        let decimals = val
            .token
            .decimals
            .clone()
            .unwrap_or(BigInt("0".into()))
            .0
            .parse::<u8>()?;

        Ok(Self {
            id: val.id.0,
            owner: val.owner,
            vault_id: val.vault_id,
            token_name: val.token.name,
            token_symbol: val.token.symbol,
            token_decimals: val.token.decimals,
            token_address: val.token.address.0,
            balance_display: format_units(balance_parsed, decimals)?,
            balance: val.balance,
        })
    }
}

impl TryIntoCsv<TokenVaultFlattened> for Vec<TokenVaultFlattened> {}
