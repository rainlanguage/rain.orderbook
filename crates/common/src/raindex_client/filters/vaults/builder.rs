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
    owners: Vec<Address>,
    hide_zero_balance: bool,
    tokens: Option<Vec<Address>>,
    chain_ids: Option<Vec<u32>>,
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
        Ok(Self::default())
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
        Ok(Self::from(filters))
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
            let address = owner_str
                .parse::<Address>()
                .map_err(|_| VaultsFilterBuilderError::InvalidAddress(owner_str.clone()))?;
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
                let address = token_str
                    .parse::<Address>()
                    .map_err(|_| VaultsFilterBuilderError::InvalidAddress(token_str.clone()))?;
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

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        fn test_new_wasm_creates_default_builder() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let filters = builder.build_wasm().unwrap();

            assert!(filters.owners.is_empty());
            assert!(!filters.hide_zero_balance);
            assert!(filters.tokens.is_none());
            assert!(filters.chain_ids.is_none());
        }

        #[wasm_bindgen_test]
        fn test_from_filters_wasm_preserves_data() {
            let original_filters = GetVaultsFilters {
                owners: vec!["0x1234567890abcdef1234567890abcdef12345678"
                    .parse()
                    .unwrap()],
                hide_zero_balance: true,
                tokens: Some(vec!["0xfedcba0987654321fedcba0987654321fedcba09"
                    .parse()
                    .unwrap()]),
                chain_ids: Some(vec![1, 137]),
            };

            let builder = VaultsFilterBuilder::from_filters_wasm(original_filters.clone()).unwrap();
            let rebuilt_filters = builder.build_wasm().unwrap();

            assert_eq!(rebuilt_filters.owners, original_filters.owners);
            assert_eq!(
                rebuilt_filters.hide_zero_balance,
                original_filters.hide_zero_balance
            );
            assert_eq!(rebuilt_filters.tokens, original_filters.tokens);
            assert_eq!(rebuilt_filters.chain_ids, original_filters.chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_set_owners_wasm_with_valid_addresses() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let owners = vec![
                "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
            ];

            let updated_builder = builder.set_owners_wasm(owners.clone()).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.owners.len(), 2);
            // Compare addresses by parsing expected values instead of string representation
            let expected_addr1: Address = "0x1234567890abcdef1234567890abcdef12345678"
                .parse()
                .unwrap();
            let expected_addr2: Address = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
                .parse()
                .unwrap();
            assert_eq!(filters.owners[0], expected_addr1);
            assert_eq!(filters.owners[1], expected_addr2);
        }

        #[wasm_bindgen_test]
        fn test_set_owners_wasm_with_invalid_address() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let invalid_owners = vec!["invalid_address".to_string()];

            let result = builder.set_owners_wasm(invalid_owners);
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_set_owners_wasm_with_empty_array() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let empty_owners = vec![];

            let updated_builder = builder.set_owners_wasm(empty_owners).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.owners.is_empty());
        }

        #[wasm_bindgen_test]
        fn test_set_hide_zero_balance_wasm_true() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_hide_zero_balance_wasm(true).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.hide_zero_balance);
        }

        #[wasm_bindgen_test]
        fn test_set_hide_zero_balance_wasm_false() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_hide_zero_balance_wasm(false).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(!filters.hide_zero_balance);
        }

        #[wasm_bindgen_test]
        fn test_set_tokens_wasm_with_valid_addresses() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let tokens = Some(vec![
                "0x1111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222".to_string(),
            ]);

            let updated_builder = builder.set_tokens_wasm(tokens).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.tokens.is_some());
            let token_vec = filters.tokens.unwrap();
            assert_eq!(token_vec.len(), 2);
            // Compare by parsing expected addresses
            let expected_token1: Address = "0x1111111111111111111111111111111111111111"
                .parse()
                .unwrap();
            let expected_token2: Address = "0x2222222222222222222222222222222222222222"
                .parse()
                .unwrap();
            assert_eq!(token_vec[0], expected_token1);
            assert_eq!(token_vec[1], expected_token2);
        }

        #[wasm_bindgen_test]
        fn test_set_tokens_wasm_with_invalid_address() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let invalid_tokens = Some(vec!["invalid_token".to_string()]);

            let result = builder.set_tokens_wasm(invalid_tokens);
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_set_tokens_wasm_with_none() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_tokens_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.tokens.is_none());
        }

        #[wasm_bindgen_test]
        fn test_set_tokens_wasm_with_empty_array() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let empty_tokens = Some(vec![]);

            let updated_builder = builder.set_tokens_wasm(empty_tokens).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            // Empty array should result in None
            assert!(filters.tokens.is_none());
        }

        #[wasm_bindgen_test]
        fn test_set_chain_ids_wasm_with_valid_ids() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let chain_ids = Some(vec![1, 137, 10]);

            let updated_builder = builder.set_chain_ids_wasm(chain_ids.clone()).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.chain_ids, chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_set_chain_ids_wasm_with_none() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_chain_ids_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.chain_ids.is_none());
        }

        #[wasm_bindgen_test]
        fn test_set_chain_ids_wasm_with_empty_array() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let empty_chain_ids = Some(vec![]);

            let updated_builder = builder.set_chain_ids_wasm(empty_chain_ids).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.chain_ids, Some(vec![]));
        }

        #[wasm_bindgen_test]
        fn test_build_wasm_returns_correct_filters() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();
            let owners = vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()];
            let tokens = Some(vec![
                "0xfedcba0987654321fedcba0987654321fedcba09".to_string()
            ]);
            let chain_ids = Some(vec![1, 137]);

            let configured_builder = builder
                .set_owners_wasm(owners)
                .unwrap()
                .set_hide_zero_balance_wasm(true)
                .unwrap()
                .set_tokens_wasm(tokens)
                .unwrap()
                .set_chain_ids_wasm(chain_ids.clone())
                .unwrap();

            let filters = configured_builder.build_wasm().unwrap();

            assert_eq!(filters.owners.len(), 1);
            assert!(filters.hide_zero_balance);
            assert!(filters.tokens.is_some());
            assert_eq!(filters.chain_ids, chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_mixed_case_addresses_wasm() {
            let builder = VaultsFilterBuilder::new_wasm().unwrap();

            // Test mixed case address (checksummed)
            let mixed_case_address = "0x1234567890aBcDeF1234567890AbCdEf12345678".to_string();
            let owners = vec![mixed_case_address.clone()];

            let updated_builder = builder.set_owners_wasm(owners).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.owners.len(), 1);
            // Compare with parsed expected address instead of string representation
            let expected_addr: Address = "0x1234567890abcdef1234567890abcdef12345678"
                .parse()
                .unwrap();
            assert_eq!(filters.owners[0], expected_addr);
        }

        #[wasm_bindgen_test]
        fn test_multiple_operations_on_same_builder_wasm() {
            let original_builder = VaultsFilterBuilder::new_wasm().unwrap();

            // First operation
            let builder1 = original_builder.set_hide_zero_balance_wasm(true).unwrap();
            let filters1 = builder1.build_wasm().unwrap();

            // Second operation on new builder (can't reuse original in WASM)
            let original_builder2 = VaultsFilterBuilder::new_wasm().unwrap();
            let builder2 = original_builder2.set_hide_zero_balance_wasm(false).unwrap();
            let filters2 = builder2.build_wasm().unwrap();

            // Verify different results
            assert!(filters1.hide_zero_balance);
            assert!(!filters2.hide_zero_balance);
        }
    }
}
