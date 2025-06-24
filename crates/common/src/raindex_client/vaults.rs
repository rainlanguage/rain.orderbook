use super::*;
use alloy::primitives::{Address, U256};
use rain_orderbook_subgraph_client::types::common::{SgErc20, SgVault};
use std::str::FromStr;
use wasm_bindgen_utils::prelude::js_sys::BigInt;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum RaindexVaultType {
    Input,
    Output,
    InputOutput,
}
impl_wasm_traits!(RaindexVaultType);

/// Represents a vault with balance and token information within a given orderbook.
///
/// A vault is a fundamental component that holds tokens and participates in order execution.
/// Each vault has a unique identifier, current balance, associated token metadata, and
/// belongs to a specific orderbook contract on the blockchain.
///
/// Vaults can serve different roles in relation to orders - they can provide tokens (input),
/// receive tokens (output), or both (input/output), depending on the trading strategy.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVault {
    vault_type: RaindexVaultType,
    id: String,
    owner: Address,
    vault_id: U256,
    balance: U256,
    token: RaindexVaultToken,
    orderbook: Address,
}

#[wasm_bindgen]
impl RaindexVault {
    #[wasm_bindgen(getter = vaultType)]
    pub fn vault_type(&self) -> RaindexVaultType {
        self.vault_type.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn owner(&self) -> String {
        self.owner.to_string()
    }
    #[cfg(target_family = "wasm")]
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.vault_id.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[cfg(not(target_family = "wasm"))]
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> String {
        self.vault_id.to_string()
    }
    #[cfg(target_family = "wasm")]
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.balance.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[cfg(not(target_family = "wasm"))]
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> String {
        self.balance.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
}

/// Token metadata associated with a vault in the Raindex system.
///
/// Contains comprehensive information about the ERC20 token held within a vault,
/// including contract address, human-readable identifiers, and decimal precision.
/// Some fields may be optional as they depend on the token's implementation and
/// whether the metadata has been successfully retrieved from the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultToken {
    id: String,
    address: Address,
    name: Option<String>,
    symbol: Option<String>,
    decimals: Option<U256>,
}
#[wasm_bindgen]
impl RaindexVaultToken {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn address(&self) -> String {
        self.address.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
    #[cfg(target_family = "wasm")]
    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> Result<Option<BigInt>, RaindexError> {
        self.decimals
            .map(|decimals| {
                BigInt::from_str(&decimals.to_string())
                    .map_err(|e| RaindexError::JsError(e.to_string().into()))
            })
            .transpose()
    }
    #[cfg(not(target_family = "wasm"))]
    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> Option<String> {
        self.decimals.map(|decimals| decimals.to_string())
    }
}

impl RaindexVault {
    pub fn try_from_sg_vault(
        vault: SgVault,
        vault_type: RaindexVaultType,
    ) -> Result<Self, RaindexError> {
        Ok(Self {
            vault_type,
            id: vault.id.0,
            owner: Address::from_str(&vault.owner.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            balance: U256::from_str(&vault.balance.0)?,
            token: vault.token.try_into()?,
            orderbook: Address::from_str(&vault.orderbook.id.0)?,
        })
    }

    pub fn with_vault_type(&self, vault_type: RaindexVaultType) -> Self {
        Self {
            vault_type,
            id: self.id.clone(),
            owner: self.owner,
            vault_id: self.vault_id,
            balance: self.balance,
            token: self.token.clone(),
            orderbook: self.orderbook,
        }
    }
}

impl TryFrom<SgErc20> for RaindexVaultToken {
    type Error = RaindexError;
    fn try_from(erc20: SgErc20) -> Result<Self, Self::Error> {
        Ok(Self {
            id: erc20.id.0,
            address: Address::from_str(&erc20.address.0)?,
            name: erc20.name,
            symbol: erc20.symbol,
            decimals: erc20
                .decimals
                .map(|decimals| U256::from_str(&decimals.0))
                .transpose()?,
        })
    }
}
