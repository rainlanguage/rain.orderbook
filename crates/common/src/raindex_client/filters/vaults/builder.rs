use super::super::traits::FilterBuilder;
use super::filter::GetVaultsFilters;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

//
// Vaults Filter Builder Error
//

#[derive(Error, Debug)]
pub enum VaultsFilterBuilderError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Owner must be a string")]
    OwnerMustBeString,
    #[error("Token must be a string")]
    TokenMustBeString,
    #[error("Chain ID must be a number")]
    ChainIdMustBeNumber,
}

impl VaultsFilterBuilderError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::InvalidAddress(addr) => format!("The address '{}' is not a valid Ethereum address. Please provide a valid hexadecimal address.", addr),
            Self::OwnerMustBeString => "Each owner must be provided as a string. Please ensure all owners are valid address strings.".to_string(),
            Self::TokenMustBeString => "Each token must be provided as a string. Please ensure all tokens are valid address strings.".to_string(),
            Self::ChainIdMustBeNumber => "Each chain ID must be provided as a number. Please ensure all chain IDs are valid integers.".to_string(),
        }
    }
}

impl From<VaultsFilterBuilderError> for WasmEncodedError {
    fn from(value: VaultsFilterBuilderError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

//
// Vaults Filter Builder
//

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct VaultsFilterBuilder {
    #[wasm_bindgen(skip)]
    pub owners: Vec<Address>,
    #[wasm_bindgen(skip)]
    pub hide_zero_balance: bool,
    #[wasm_bindgen(skip)]
    pub tokens: Option<Vec<Address>>,
    #[wasm_bindgen(skip)]
    pub chain_ids: Option<Vec<u32>>,
}

#[wasm_export]
impl VaultsFilterBuilder {
    /// Creates a new VaultsFilterBuilder with default values.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.new();
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "A new VaultsFilterBuilder instance with default values"
    )]
    pub fn new_wasm() -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        Ok(Self {
            owners: Vec::new(),
            hide_zero_balance: false,
            tokens: None,
            chain_ids: None,
        })
    }

    /// Creates a VaultsFilterBuilder from existing filters.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.fromFilters(existingFilters);
    /// ```
    #[wasm_export(
        js_name = "fromFilters",
        preserve_js_class,
        return_description = "A VaultsFilterBuilder instance created from existing filters"
    )]
    pub fn from_filters_wasm(
        filters: GetVaultsFilters,
    ) -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        Ok(Self {
            owners: filters.owners,
            hide_zero_balance: filters.hide_zero_balance,
            tokens: filters.tokens,
            chain_ids: filters.chain_ids,
        })
    }

    /// Sets the owners for the filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.new()
    ///     .setOwners(["0x1234567890abcdef1234567890abcdef12345678"]);
    /// ```
    #[wasm_export(
        js_name = "setOwners",
        preserve_js_class,
        return_description = "A new VaultsFilterBuilder instance with updated owners"
    )]
    pub fn set_owners_wasm(
        self,
        #[wasm_export(
            param_description = "Array of owner addresses",
            unchecked_param_type = "Address[]"
        )]
        owners: Vec<String>,
    ) -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        let mut rust_owners = Vec::new();
        for owner_str in owners {
            let address = owner_str.parse::<Address>().map_err(|e| {
                VaultsFilterBuilderError::InvalidAddress(format!("Invalid address: {}", e))
            })?;
            rust_owners.push(address);
        }
        let next = self.clone().set_owners(rust_owners);
        Ok(next)
    }

    /// Sets whether to hide zero balance vaults.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.new()
    ///     .setHideZeroBalance(true);
    /// ```
    #[wasm_export(
        js_name = "setHideZeroBalance",
        preserve_js_class,
        return_description = "A new VaultsFilterBuilder instance with updated hide zero balance setting"
    )]
    pub fn set_hide_zero_balance_wasm(
        self,
        hide_zero_balance: bool,
    ) -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        let next = self.clone().set_hide_zero_balance(hide_zero_balance);
        Ok(next)
    }

    /// Sets the tokens for the filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.new()
    ///     .setTokens(["0x1111111111111111111111111111111111111111"]);
    /// ```
    #[wasm_export(
        js_name = "setTokens",
        preserve_js_class,
        return_description = "A new VaultsFilterBuilder instance with updated tokens"
    )]
    pub fn set_tokens_wasm(
        self,
        #[wasm_export(
            param_description = "Optional array of token addresses",
            unchecked_param_type = "Address[] | undefined"
        )]
        tokens: Option<Vec<String>>,
    ) -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        let mut tokens_list = Vec::new();
        if let Some(tokens_vec) = tokens {
            for token_str in tokens_vec {
                let address = token_str.parse::<Address>().map_err(|e| {
                    VaultsFilterBuilderError::InvalidAddress(format!(
                        "Invalid token address: {}",
                        e
                    ))
                })?;
                tokens_list.push(address);
            }
        }
        let next_tokens = if tokens_list.is_empty() {
            None
        } else {
            Some(tokens_list)
        };
        let next = self.clone().set_tokens(next_tokens);
        Ok(next)
    }

    /// Sets the chain IDs for the filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = VaultsFilterBuilder.new()
    ///     .setChainIds([1, 137, 10]);
    /// ```
    #[wasm_export(
        js_name = "setChainIds",
        preserve_js_class,
        return_description = "A new VaultsFilterBuilder instance with updated chain IDs"
    )]
    pub fn set_chain_ids_wasm(
        self,
        #[wasm_export(
            param_description = "Optional array of chain IDs",
            unchecked_param_type = "number[] | undefined"
        )]
        chain_ids: Option<Vec<u32>>,
    ) -> Result<VaultsFilterBuilder, VaultsFilterBuilderError> {
        let next = self.clone().set_chain_ids(chain_ids);
        Ok(next)
    }

    /// Builds the filter into a GetVaultsFilters instance.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const filters = VaultsFilterBuilder.new()
    ///     .setOwners(["0x1234567890abcdef1234567890abcdef12345678"])
    ///     .setHideZeroBalance(true)
    ///     .build();
    /// ```
    #[wasm_export(
        js_name = "build",
        preserve_js_class,
        return_description = "A GetVaultsFilters instance with the configured values"
    )]
    pub fn build_wasm(self) -> Result<GetVaultsFilters, VaultsFilterBuilderError> {
        Ok(self.build())
    }
}

// Implementation for internal Rust usage
impl VaultsFilterBuilder {
    pub fn set_owners(mut self, owners: Vec<Address>) -> Self {
        self.owners = owners;
        self
    }

    pub fn set_hide_zero_balance(mut self, hide_zero_balance: bool) -> Self {
        self.hide_zero_balance = hide_zero_balance;
        self
    }

    pub fn set_tokens(mut self, tokens: Option<Vec<Address>>) -> Self {
        self.tokens = tokens;
        self
    }

    pub fn set_chain_ids(mut self, chain_ids: Option<Vec<u32>>) -> Self {
        self.chain_ids = chain_ids;
        self
    }

    pub fn build(self) -> GetVaultsFilters {
        GetVaultsFilters {
            owners: self.owners,
            hide_zero_balance: self.hide_zero_balance,
            tokens: self.tokens,
            chain_ids: self.chain_ids,
        }
    }
}

impl FilterBuilder for VaultsFilterBuilder {
    type Output = GetVaultsFilters;

    fn build(self) -> Self::Output {
        // Call the internal build method directly to avoid conflict
        self.build()
    }
}

impl From<VaultsFilterBuilder> for GetVaultsFilters {
    fn from(builder: VaultsFilterBuilder) -> Self {
        // Call the internal build logic directly to avoid conflict
        builder.build()
    }
}

impl From<GetVaultsFilters> for VaultsFilterBuilder {
    fn from(filters: GetVaultsFilters) -> Self {
        Self {
            owners: filters.owners,
            hide_zero_balance: filters.hide_zero_balance,
            tokens: filters.tokens,
            chain_ids: filters.chain_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let builder = VaultsFilterBuilder::default()
            .set_owners(vec![Address::ZERO])
            .set_hide_zero_balance(true);

        let filters = builder.build();
        assert_eq!(filters.owners, vec![Address::ZERO]);
        assert!(filters.hide_zero_balance);
    }

    #[test]
    fn test_from_conversion() {
        let filters = GetVaultsFilters {
            owners: vec![Address::ZERO],
            hide_zero_balance: true,
            tokens: None,
            chain_ids: Some(vec![1, 2]),
        };

        let builder: VaultsFilterBuilder = filters.clone().into();
        let converted_back: GetVaultsFilters = builder.into();

        assert_eq!(filters.owners, converted_back.owners);
        assert_eq!(filters.hide_zero_balance, converted_back.hide_zero_balance);
        assert_eq!(filters.tokens, converted_back.tokens);
        assert_eq!(filters.chain_ids, converted_back.chain_ids);
    }
}
