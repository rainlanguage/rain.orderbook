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
}
