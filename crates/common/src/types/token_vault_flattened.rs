use crate::csv::TryIntoCsv;
use alloy::primitives::{utils::format_units, U256};
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: SgBytes,
    pub vault_id: SgBigInt,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<SgBigInt>,
    pub token_address: String,
    pub balance_display: String,
    pub balance: SgBigInt,
}

impl TryFrom<SgVault> for TokenVaultFlattened {
    type Error = FlattenError;

    fn try_from(val: SgVault) -> Result<Self, Self::Error> {
        let balance_parsed = val.balance.0.parse::<U256>()?;
        let decimals = val
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
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
