use super::super::traits::FilterBuilder;
use super::filter::GetOrdersFilters;
use alloy::primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

//
// Orders Filter Builder Error
//

#[derive(Error, Debug)]
pub enum OrdersFilterBuilderError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid order hash: {0}")]
    InvalidOrderHash(String),
    #[error("Owner must be a string")]
    OwnerMustBeString,
    #[error("Token must be a string")]
    TokenMustBeString,
    #[error("Order hash must be a string")]
    OrderHashMustBeString,
    #[error("Active must be a boolean")]
    ActiveMustBeBoolean,
}

impl OrdersFilterBuilderError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::InvalidAddress(addr) => format!("The address '{}' is not valid.", addr),
            Self::InvalidOrderHash(hash) => format!("The order hash '{}' is not valid.", hash),
            Self::OwnerMustBeString => "Each owner must be provided as Address string. Please ensure all owners are valid address strings.".to_string(),
            Self::TokenMustBeString => "Each token must be provided as Address string. Please ensure all tokens are valid address strings.".to_string(),
            Self::OrderHashMustBeString => "Order hash must be provided as Address string. Please ensure it is a valid hexadecimal hash string.".to_string(),
            Self::ActiveMustBeBoolean => "Active filter must be provided as a boolean value (true or false).".to_string(),
        }
    }
}

impl From<OrdersFilterBuilderError> for WasmEncodedError {
    fn from(value: OrdersFilterBuilderError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

//
// Orders Filter Builder
//

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct OrdersFilterBuilder {
    owners: Vec<Address>,
    active: Option<bool>,
    order_hash: Option<Bytes>,
    tokens: Option<Vec<Address>>,
    chain_ids: Option<Vec<u32>>,
}

#[wasm_export]
impl OrdersFilterBuilder {
    /// Creates a new OrdersFilterBuilder with default values.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new();
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with default values"
    )]
    pub fn new_wasm() -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        Ok(Self::default())
    }

    /// Creates an OrdersFilterBuilder from existing filters.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.fromFilters(existingFilters);
    /// ```
    #[wasm_export(
        js_name = "fromFilters",
        preserve_js_class,
        return_description = "An OrdersFilterBuilder instance created from existing filters"
    )]
    pub fn from_filters_wasm(
        filters: GetOrdersFilters,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        Ok(Self::from(filters))
    }

    /// Sets the owners for the filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new()
    ///     .setOwners(["0x1234567890abcdef1234567890abcdef12345678"]);
    /// ```
    #[wasm_export(
        js_name = "setOwners",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with updated owners"
    )]
    pub fn set_owners_wasm(
        self,
        #[wasm_export(
            param_description = "Array of owner addresses",
            unchecked_param_type = "Address[]"
        )]
        owners: Vec<String>,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        let mut rust_owners = Vec::new();
        for owner_str in owners {
            let address = owner_str
                .parse::<Address>()
                .map_err(|_| OrdersFilterBuilderError::InvalidAddress(owner_str.clone()))?;
            rust_owners.push(address);
        }
        let next = self.clone().set_owners(rust_owners);
        Ok(next)
    }

    /// Sets the active filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new()
    ///     .setActive(true);
    /// ```
    #[wasm_export(
        js_name = "setActive",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with updated active filter"
    )]
    pub fn set_active_wasm(
        self,
        active: Option<bool>,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        let next = self.clone().set_active(active);
        Ok(next)
    }

    /// Sets the order hash filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new()
    ///     .setOrderHash("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    /// ```
    #[wasm_export(
        js_name = "setOrderHash",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with updated order hash filter"
    )]
    pub fn set_order_hash_wasm(
        self,
        #[wasm_export(
            param_description = "Order hash as hexadecimal string",
            unchecked_param_type = "Hex | undefined"
        )]
        order_hash: Option<String>,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        let rust_hash = match order_hash {
            Some(hash_str) => Some(
                hash_str
                    .parse::<Bytes>()
                    .map_err(|_| OrdersFilterBuilderError::InvalidOrderHash(hash_str))?,
            ),
            None => None,
        };
        let next = self.clone().set_order_hash(rust_hash);
        Ok(next)
    }

    /// Sets the tokens filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new()
    ///     .setTokens(["0xfedcba0987654321fedcba0987654321fedcba09"]);
    /// ```
    #[wasm_export(
        js_name = "setTokens",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with updated tokens filter"
    )]
    pub fn set_tokens_wasm(
        self,
        #[wasm_export(
            param_description = "Array of token addresses",
            unchecked_param_type = "Address[] | undefined"
        )]
        tokens: Option<Vec<String>>,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        let rust_tokens = match tokens {
            Some(token_strs) => {
                let mut rust_tokens = Vec::new();
                for token_str in token_strs {
                    let address = token_str
                        .parse::<Address>()
                        .map_err(|_| OrdersFilterBuilderError::InvalidAddress(token_str.clone()))?;
                    rust_tokens.push(address);
                }
                Some(rust_tokens)
            }
            None => None,
        };
        let next = self.clone().set_tokens(rust_tokens);
        Ok(next)
    }

    /// Sets the chain IDs filter.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const builder = OrdersFilterBuilder.new()
    ///     .setChainIds([1, 137, 42161]);
    /// ```
    #[wasm_export(
        js_name = "setChainIds",
        preserve_js_class,
        return_description = "A new OrdersFilterBuilder instance with updated chain IDs filter"
    )]
    pub fn set_chain_ids_wasm(
        self,
        #[wasm_export(
            param_description = "Array of chain IDs",
            unchecked_param_type = "ChainIds | undefined"
        )]
        chain_ids: Option<Vec<u32>>,
    ) -> Result<OrdersFilterBuilder, OrdersFilterBuilderError> {
        let next = self.clone().set_chain_ids(chain_ids);
        Ok(next)
    }

    /// Builds the final GetOrdersFilters from the builder.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const filters = OrdersFilterBuilder.new()
    ///     .setOwners(["0x1234567890abcdef1234567890abcdef12345678"])
    ///     .setActive(true)
    ///     .build();
    /// ```
    #[wasm_export(
        js_name = "build",
        preserve_js_class,
        return_description = "The built GetOrdersFilters"
    )]
    pub fn build_wasm(self) -> Result<GetOrdersFilters, OrdersFilterBuilderError> {
        Ok(self.build())
    }
}

impl OrdersFilterBuilder {
    /// Sets the owners for the filter.
    pub fn set_owners(mut self, owners: Vec<Address>) -> Self {
        self.owners = owners;
        self
    }

    /// Sets the active filter.
    pub fn set_active(mut self, active: Option<bool>) -> Self {
        self.active = active;
        self
    }

    /// Sets the order hash filter.
    pub fn set_order_hash(mut self, order_hash: Option<Bytes>) -> Self {
        self.order_hash = order_hash;
        self
    }

    /// Sets the tokens filter.
    pub fn set_tokens(mut self, tokens: Option<Vec<Address>>) -> Self {
        self.tokens = tokens;
        self
    }

    /// Sets the chain IDs filter.
    pub fn set_chain_ids(mut self, chain_ids: Option<Vec<u32>>) -> Self {
        self.chain_ids = chain_ids;
        self
    }

    /// Builds the final GetOrdersFilters from the builder.
    pub fn build(self) -> GetOrdersFilters {
        GetOrdersFilters {
            owners: self.owners,
            active: self.active,
            order_hash: self.order_hash,
            tokens: self.tokens,
            chain_ids: self.chain_ids,
        }
    }
}

impl From<GetOrdersFilters> for OrdersFilterBuilder {
    fn from(filters: GetOrdersFilters) -> Self {
        Self {
            owners: filters.owners,
            active: filters.active,
            order_hash: filters.order_hash,
            tokens: filters.tokens,
            chain_ids: filters.chain_ids,
        }
    }
}

impl FilterBuilder for OrdersFilterBuilder {
    type Output = GetOrdersFilters;

    fn build(self) -> Self::Output {
        self.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orders_filter_builder_default() {
        let builder = OrdersFilterBuilder::default();
        let filters = builder.build();

        assert!(filters.owners.is_empty());
        assert!(filters.active.is_none());
        assert!(filters.order_hash.is_none());
        assert!(filters.tokens.is_none());
    }

    #[test]
    fn test_orders_filter_builder_set_owners() {
        let builder = OrdersFilterBuilder::default();
        let owner = "0x1234567890abcdef1234567890abcdef12345678"
            .parse::<Address>()
            .unwrap();
        let filters = builder.set_owners(vec![owner]).build();

        assert_eq!(filters.owners.len(), 1);
        assert_eq!(filters.owners[0], owner);
    }

    #[test]
    fn test_orders_filter_builder_set_active() {
        let builder = OrdersFilterBuilder::default();
        let filters = builder.set_active(Some(true)).build();

        assert_eq!(filters.active, Some(true));
    }

    #[test]
    fn test_orders_filter_builder_from_filters() {
        let original_filters = GetOrdersFilters {
            owners: vec!["0x1234567890abcdef1234567890abcdef12345678"
                .parse()
                .unwrap()],
            active: Some(true),
            order_hash: None,
            tokens: None,
            chain_ids: Some(vec![1, 137]),
        };

        let builder = OrdersFilterBuilder::from(original_filters.clone());
        let rebuilt_filters = builder.build();

        assert_eq!(rebuilt_filters.owners, original_filters.owners);
        assert_eq!(rebuilt_filters.active, original_filters.active);
        assert_eq!(rebuilt_filters.order_hash, original_filters.order_hash);
        assert_eq!(rebuilt_filters.tokens, original_filters.tokens);
        assert_eq!(rebuilt_filters.chain_ids, original_filters.chain_ids);
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        fn test_orders_new_wasm_creates_default_builder() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let filters = builder.build_wasm().unwrap();

            assert!(filters.owners.is_empty());
            assert!(filters.active.is_none());
            assert!(filters.order_hash.is_none());
            assert!(filters.tokens.is_none());
            assert!(filters.chain_ids.is_none());
        }

        #[wasm_bindgen_test]
        fn test_orders_from_filters_wasm_preserves_data() {
            let original_filters = GetOrdersFilters {
                owners: vec!["0x1234567890abcdef1234567890abcdef12345678"
                    .parse()
                    .unwrap()],
                active: Some(true),
                order_hash: Some(
                    "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                        .parse()
                        .unwrap(),
                ),
                tokens: Some(vec!["0xfedcba0987654321fedcba0987654321fedcba09"
                    .parse()
                    .unwrap()]),
                chain_ids: Some(vec![1, 137]),
            };

            let builder = OrdersFilterBuilder::from_filters_wasm(original_filters.clone()).unwrap();
            let rebuilt_filters = builder.build_wasm().unwrap();

            assert_eq!(rebuilt_filters.owners, original_filters.owners);
            assert_eq!(rebuilt_filters.active, original_filters.active);
            assert_eq!(rebuilt_filters.order_hash, original_filters.order_hash);
            assert_eq!(rebuilt_filters.tokens, original_filters.tokens);
            assert_eq!(rebuilt_filters.chain_ids, original_filters.chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_orders_set_owners_wasm_with_valid_addresses() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
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
        fn test_orders_set_owners_wasm_with_invalid_address() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let invalid_owners = vec!["invalid_address".to_string()];

            let result = builder.set_owners_wasm(invalid_owners);
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_owners_wasm_with_empty_array() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let empty_owners = vec![];

            let updated_builder = builder.set_owners_wasm(empty_owners).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.owners.is_empty());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_active_wasm_true() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_active_wasm(Some(true)).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.active, Some(true));
        }

        #[wasm_bindgen_test]
        fn test_orders_set_active_wasm_false() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_active_wasm(Some(false)).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.active, Some(false));
        }

        #[wasm_bindgen_test]
        fn test_orders_set_active_wasm_none() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_active_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.active, None);
        }

        #[wasm_bindgen_test]
        fn test_orders_set_order_hash_wasm_valid() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let hash =
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string();

            let updated_builder = builder.set_order_hash_wasm(Some(hash.clone())).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.order_hash.is_some());
            // Compare by parsing the expected hash instead of string representation
            let expected_hash: Bytes = hash.parse().unwrap();
            assert_eq!(filters.order_hash.unwrap(), expected_hash);
        }

        #[wasm_bindgen_test]
        fn test_orders_set_order_hash_wasm_invalid() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let invalid_hash = "invalid_hash".to_string();

            let result = builder.set_order_hash_wasm(Some(invalid_hash));
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_order_hash_wasm_none() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_order_hash_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.order_hash.is_none());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_tokens_wasm_with_valid_addresses() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let tokens = Some(vec![
                "0xfedcba0987654321fedcba0987654321fedcba09".to_string(),
                "0x1111111111111111111111111111111111111111".to_string(),
            ]);

            let updated_builder = builder.set_tokens_wasm(tokens).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.tokens.is_some());
            let token_vec = filters.tokens.unwrap();
            assert_eq!(token_vec.len(), 2);
            // Compare by parsing expected addresses
            let expected_token1: Address = "0xfedcba0987654321fedcba0987654321fedcba09"
                .parse()
                .unwrap();
            let expected_token2: Address = "0x1111111111111111111111111111111111111111"
                .parse()
                .unwrap();
            assert_eq!(token_vec[0], expected_token1);
            assert_eq!(token_vec[1], expected_token2);
        }

        #[wasm_bindgen_test]
        fn test_orders_set_tokens_wasm_with_invalid_address() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let invalid_tokens = Some(vec!["invalid_token".to_string()]);

            let result = builder.set_tokens_wasm(invalid_tokens);
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_tokens_wasm_with_none() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_tokens_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.tokens.is_none());
        }

        #[wasm_bindgen_test]
        fn test_orders_set_chain_ids_wasm_with_valid_ids() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let chain_ids = Some(vec![1, 137, 42161]);

            let updated_builder = builder.set_chain_ids_wasm(chain_ids.clone()).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert_eq!(filters.chain_ids, chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_orders_set_chain_ids_wasm_with_none() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let updated_builder = builder.set_chain_ids_wasm(None).unwrap();
            let filters = updated_builder.build_wasm().unwrap();

            assert!(filters.chain_ids.is_none());
        }

        #[wasm_bindgen_test]
        fn test_orders_build_wasm_returns_correct_filters() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();
            let owners = vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()];
            let tokens = Some(vec![
                "0xfedcba0987654321fedcba0987654321fedcba09".to_string()
            ]);
            let chain_ids = Some(vec![1, 137]);

            let configured_builder = builder
                .set_owners_wasm(owners)
                .unwrap()
                .set_active_wasm(Some(true))
                .unwrap()
                .set_tokens_wasm(tokens)
                .unwrap()
                .set_chain_ids_wasm(chain_ids.clone())
                .unwrap();

            let filters = configured_builder.build_wasm().unwrap();

            assert_eq!(filters.owners.len(), 1);
            assert_eq!(filters.active, Some(true));
            assert!(filters.tokens.is_some());
            assert_eq!(filters.chain_ids, chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_orders_complex_filter_combination_wasm() {
            let builder = OrdersFilterBuilder::new_wasm().unwrap();

            let owners = vec![
                "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
            ];
            let tokens = Some(vec![
                "0xfedcba0987654321fedcba0987654321fedcba09".to_string(),
                "0x2222222222222222222222222222222222222222".to_string(),
            ]);
            let hash =
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string();
            let chain_ids = Some(vec![1, 10, 137, 42161]);

            let configured_builder = builder
                .set_owners_wasm(owners)
                .unwrap()
                .set_active_wasm(Some(false))
                .unwrap()
                .set_order_hash_wasm(Some(hash.clone()))
                .unwrap()
                .set_tokens_wasm(tokens)
                .unwrap()
                .set_chain_ids_wasm(chain_ids.clone())
                .unwrap();

            let filters = configured_builder.build_wasm().unwrap();

            assert_eq!(filters.owners.len(), 2);
            assert_eq!(filters.active, Some(false));
            assert!(filters.order_hash.is_some());
            // Compare the hash by parsing both sides
            let expected_hash: Bytes = hash.parse().unwrap();
            assert_eq!(filters.order_hash.unwrap(), expected_hash);
            assert!(filters.tokens.is_some());
            assert_eq!(filters.tokens.unwrap().len(), 2);
            assert_eq!(filters.chain_ids, chain_ids);
        }

        #[wasm_bindgen_test]
        fn test_orders_multiple_operations_immutability_wasm() {
            let original_builder = OrdersFilterBuilder::new_wasm().unwrap();

            // First operation
            let builder1 = original_builder.set_active_wasm(Some(true)).unwrap();
            let filters1 = builder1.build_wasm().unwrap();

            // Second operation on new builder (can't reuse original in WASM)
            let original_builder2 = OrdersFilterBuilder::new_wasm().unwrap();
            let builder2 = original_builder2.set_active_wasm(Some(false)).unwrap();
            let filters2 = builder2.build_wasm().unwrap();

            // Verify different results
            assert_eq!(filters1.active, Some(true));
            assert_eq!(filters2.active, Some(false));
        }
    }
}
