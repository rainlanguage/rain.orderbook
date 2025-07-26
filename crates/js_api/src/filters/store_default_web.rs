use super::errors::PersistentFilterStoreError;
use super::store_localstorage::LocalStorageFilterStore;
use super::store_urlparams::URLParamsFilterStore;
use super::traits::PersistentFilterStore;
use anyhow;
use rain_orderbook_common::raindex_client::filters::{
    store_basic::BasicFilterStore, traits::FilterStore, vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};
use serde::{Deserialize, Serialize};

#[cfg(target_family = "wasm")]
use js_sys;
#[cfg(target_family = "wasm")]
use serde_wasm_bindgen;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

/// Default web filter store composition for browsers.
///
/// This store provides a complete filtering solution for web applications:
/// - URLParams layer: Saves/loads filters from URL search parameters (for sharing)
/// - LocalStorage layer: Persists filters across browser sessions
/// - BasicFilterStore layer: Core filter logic
///
/// Priority order:
/// 1. URL parameters (highest - for sharing links)
/// 2. LocalStorage (fallback - for user persistence)
/// 3. Default values (lowest - initial state)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct DefaultWebFilterStore {
    inner: URLParamsFilterStore<LocalStorageFilterStore<BasicFilterStore>>,
}

impl DefaultWebFilterStore {
    /// Create a new default web filter store.
    ///
    /// # Arguments
    /// * `key` - LocalStorage key for persisting filters
    ///
    /// # Example
    /// ```rust
    /// let store = DefaultWebFilterStore::new("my-app-filters")?;
    /// ```
    pub fn new(key: &str) -> Result<Self, PersistentFilterStoreError> {
        let inner = URLParamsFilterStore::new(key)?;
        Ok(Self { inner })
    }
}

impl FilterStore for DefaultWebFilterStore {
    fn get_vaults(&self) -> GetVaultsFilters {
        self.inner.get_vaults()
    }

    fn set_vaults(&mut self, filters: GetVaultsFilters) {
        self.inner.set_vaults(filters);
    }

    fn update_vaults<F>(&mut self, update_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder,
    {
        self.inner.update_vaults(update_fn)?;
        Ok(())
    }
}

impl PersistentFilterStore for DefaultWebFilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        self.inner.load()
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        self.inner.save()
    }
}

// WASM bindings
#[cfg(target_family = "wasm")]
#[wasm_export]
impl DefaultWebFilterStore {
    #[wasm_export(
        js_name = "create",
        preserve_js_class,
        return_description = "Creates a new DefaultWebFilterStore instance"
    )]
    pub fn create(key: &str) -> Result<DefaultWebFilterStore, PersistentFilterStoreError> {
        DefaultWebFilterStore::new(key)
    }

    #[wasm_export(
        js_name = "getVaults",
        preserve_js_class,
        return_description = "Returns current vault filters"
    )]
    pub fn get_vaults_wasm(&self) -> Result<GetVaultsFilters, PersistentFilterStoreError> {
        Ok(self.get_vaults())
    }

    #[wasm_export(
        js_name = "setVaults",
        preserve_js_class,
        return_description = "Sets vault filters without side-effects and returns updated store instance"
    )]
    pub fn set_vaults_wasm(
        self,
        filters: GetVaultsFilters,
    ) -> Result<DefaultWebFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.set_vaults(filters);
        Ok(store)
    }

    #[wasm_export(
        js_name = "updateVaultsWithBuilder",
        preserve_js_class,
        return_description = "Updates vault filters using a builder pattern, returns updated store instance"
    )]
    pub fn update_vaults_with_builder_wasm(
        self,
        #[wasm_export(
            param_description = "Builder function that receives current VaultsFilterBuilder and returns modified one"
        )]
        builder_update: js_sys::Function,
    ) -> Result<DefaultWebFilterStore, PersistentFilterStoreError> {
        use wasm_bindgen::JsValue;

        // Get current builder and serialize for JavaScript
        let current_filters = self.get_vaults();
        let current_builder = VaultsFilterBuilder::from_filters(current_filters)
            .map_err(|e| PersistentFilterStoreError::LoadError(e.to_string()))?;

        let builder_js = serde_wasm_bindgen::to_value(&current_builder).map_err(|e| {
            PersistentFilterStoreError::SaveError(format!("Failed to serialize builder: {}", e))
        })?;

        // Call JavaScript function
        let updated_builder_js =
            builder_update
                .call1(&JsValue::NULL, &builder_js)
                .map_err(|e| {
                    PersistentFilterStoreError::SaveError(format!(
                        "Builder function failed: {:?}",
                        e
                    ))
                })?;

        // Deserialize result
        let updated_builder: VaultsFilterBuilder =
            serde_wasm_bindgen::from_value(updated_builder_js).map_err(|e| {
                PersistentFilterStoreError::SaveError(format!(
                    "Failed to deserialize builder: {}",
                    e
                ))
            })?;

        // Use native update_vaults with a closure that returns the updated builder
        let mut next = self.clone();
        next.update_vaults(|_current_builder| updated_builder)
            .map_err(|e| PersistentFilterStoreError::SaveError(e.to_string()))?;

        Ok(next)
    }

    #[wasm_export(
        js_name = "save",
        unchecked_return_type = "void",
        return_description = "Saves current filters to persistent storage (localStorage and URL params)"
    )]
    pub fn save_wasm(&self) -> Result<(), PersistentFilterStoreError> {
        self.save()
    }

    #[wasm_export(
        js_name = "load",
        preserve_js_class,
        return_description = "Loads filters from persistent storage and returns updated store instance"
    )]
    pub fn load_wasm(self) -> Result<DefaultWebFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.load()?;
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::test_utils::filters_equal;
    use alloy::primitives::Address;
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_set_and_get_vaults() {
        let key = "test_set_get_vaults";

        if let Ok(mut store) = DefaultWebFilterStore::new(key) {
            let original_filters = store.get_vaults();

            // Set new filters with non-default values
            let owner = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
            let new_filters = GetVaultsFilters {
                owners: vec![owner],
                hide_zero_balance: true,
                tokens: None,
                chain_ids: Some(vec![1, 137]),
            };
            store.set_vaults(new_filters.clone());

            let retrieved_filters = store.get_vaults();

            // Verify that the filters actually changed
            assert!(
                !filters_equal(&original_filters, &retrieved_filters),
                "Filters should have changed from default"
            );
            assert!(
                filters_equal(&new_filters, &retrieved_filters),
                "Set and retrieved filters should match"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_update_vaults_with_builder() {
        let key = "test_update_builder";

        if let Ok(mut store) = DefaultWebFilterStore::new(key) {
            let original_filters = store.get_vaults();

            use alloy::primitives::Address;
            use std::str::FromStr;
            let owner = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

            let update_result = store.update_vaults(|builder| {
                builder.set_owners(vec![owner]).set_hide_zero_balance(true)
            });

            assert!(update_result.is_ok(), "Update operation should succeed");

            let updated_filters = store.get_vaults();

            // Verify that the filters actually changed
            assert!(
                !filters_equal(&original_filters, &updated_filters),
                "Filters should have changed after update"
            );
            assert_eq!(updated_filters.owners.len(), 1, "Should have one owner");
            assert_eq!(updated_filters.owners[0], owner, "Owner should match");
            assert!(
                updated_filters.hide_zero_balance,
                "hide_zero_balance should be true"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_serialization_roundtrip() {
        let key = "test_serialization";

        if let Ok(store) = DefaultWebFilterStore::new(key) {
            // Test that store can be serialized and deserialized
            let serialized = serde_json::to_string(&store);
            assert!(serialized.is_ok(), "Store should serialize successfully");

            if let Ok(json) = serialized {
                let deserialized: Result<DefaultWebFilterStore, _> = serde_json::from_str(&json);
                assert!(
                    deserialized.is_ok(),
                    "Store should deserialize successfully"
                );
            }
        }
    }

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_get_vaults() {
        let key = "test_wasm_get_vaults";

        if let Ok(store) = DefaultWebFilterStore::create(key) {
            let result = store.get_vaults_wasm();
            assert!(result.is_ok(), "get_vaults_wasm should return Ok");
        }
    }

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_update_vaults() {
        let key = "test_wasm_update_vaults";

        if let Ok(store) = DefaultWebFilterStore::create(key) {
            let original_filters = store.get_vaults_wasm().unwrap();
            // Create a JavaScript function that modifies the builder
            use js_sys::Function;

            let js_code = r#"
                function(builder) {
                    // Create new owners array with a test address
                    const newOwners = ["0x1234567890abcdef1234567890abcdef12345678"];
                    
                    builder.setOwners(newOwners)
                        .setHideZeroBalance(true)
                        .setChainIds([1, 137]);
                }
            "#;
            // Same as above
            let expected_filters = VaultsFilterBuilder::from_filters(original_filters.clone())
                .unwrap()
                .set_owners(vec![Address::from_str(
                    "0x1234567890abcdef1234567890abcdef12345678",
                )
                .unwrap()])
                .set_hide_zero_balance(true)
                .set_chain_ids(Some(vec![1, 137]))
                .build();

            let js_function = Function::new_no_args(js_code);

            // Test update_vaults_with_builder_wasm
            let updated_store = store.update_vaults_with_builder_wasm(js_function).unwrap();
            let updated_filters = updated_store.get_vaults_wasm().unwrap();

            // Verify that the filters actually changed
            assert!(
                !filters_equal(&original_filters, &updated_filters),
                "Filters should have changed from original"
            );
            assert!(
                filters_equal(&expected_filters, &updated_filters),
                "Filters changed to expected values"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_store_state_consistency() {
        let key = "test_state_consistency";

        if let Ok(mut store) = DefaultWebFilterStore::new(key) {
            // Test that multiple operations maintain consistent state
            let _original = store.get_vaults();

            // Multiple updates should be consistent
            let _ = store.update_vaults(|builder| builder);
            let after_update1 = store.get_vaults();

            let _ = store.update_vaults(|builder| builder);
            let after_update2 = store.get_vaults();

            // State should remain consistent through updates
            assert!(filters_equal(&after_update1, &after_update2));
        }
    }

    #[wasm_bindgen_test]
    fn test_store_debug_implementation() {
        let key = "test_debug";

        if let Ok(store) = DefaultWebFilterStore::new(key) {
            // Test that Debug trait is properly implemented
            let debug_output = format!("{:?}", store);
            assert!(!debug_output.is_empty(), "Debug output should not be empty");
            assert!(
                debug_output.contains("DefaultWebFilterStore"),
                "Debug should contain struct name"
            );
        }
    }

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_vaults_filter_builder_wasm() {
        // Test VaultsFilterBuilder WASM methods
        use rain_orderbook_common::raindex_client::filters::vaults_builder::VaultsFilterBuilder;

        let builder_result = VaultsFilterBuilder::new_wasm();
        assert!(builder_result.is_ok(), "Should create builder successfully");

        if let Ok(builder) = builder_result {
            // Test setOwners method
            let owners = vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()];
            let builder_with_owners = builder.set_owners_wasm(owners).unwrap();

            // Test setHideZeroBalance method
            let builder_with_hide = builder_with_owners
                .set_hide_zero_balance_wasm(true)
                .unwrap();

            // Test setChainIds method
            let chain_ids = Some(vec![1, 137]);
            let builder_with_chains = builder_with_hide.set_chain_ids_wasm(chain_ids).unwrap();

            // Test build method
            let filters = builder_with_chains.build_wasm().unwrap();

            // Verify the built filters
            assert_eq!(filters.owners.len(), 1, "Should have one owner");
            assert!(filters.hide_zero_balance, "Should hide zero balance");
            assert_eq!(
                filters.chain_ids,
                Some(vec![1, 137]),
                "Should have correct chain IDs"
            );
        }
    }
}
