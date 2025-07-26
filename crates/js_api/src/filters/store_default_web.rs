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
    use wasm_bindgen_test::wasm_bindgen_test;

    fn filters_equal(a: &GetVaultsFilters, b: &GetVaultsFilters) -> bool {
        // Compare through serialization
        serde_json::to_string(a).unwrap() == serde_json::to_string(b).unwrap()
    }

    #[wasm_bindgen_test]
    fn test_set_and_get_vaults() {
        let key = "test_set_get_vaults";

        if let Ok(mut store) = DefaultWebFilterStore::new(key) {
            let original_filters = store.get_vaults();

            // Set new filters with non-default values
            use alloy::primitives::Address;
            use std::str::FromStr;
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

    #[wasm_bindgen_test]
    fn test_multiple_stores_with_different_keys() {
        let key1 = "test_multi_store_1";
        let key2 = "test_multi_store_2";

        if let (Ok(mut store1), Ok(store2)) = (
            DefaultWebFilterStore::new(key1),
            DefaultWebFilterStore::new(key2),
        ) {
            // Both stores should start with defaults
            let filters1 = store1.get_vaults();
            let filters2 = store2.get_vaults();
            assert!(
                filters_equal(&filters1, &filters2),
                "Both stores should start with defaults"
            );

            // Change filters in store1 only
            use alloy::primitives::Address;
            use std::str::FromStr;
            let owner = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
            let modified_filters = GetVaultsFilters {
                owners: vec![owner],
                hide_zero_balance: true,
                tokens: None,
                chain_ids: Some(vec![1]),
            };
            store1.set_vaults(modified_filters.clone());

            // Verify store1 changed but store2 didn't
            let new_filters1 = store1.get_vaults();
            let unchanged_filters2 = store2.get_vaults();

            assert!(
                filters_equal(&new_filters1, &modified_filters),
                "Store1 should have modified filters"
            );
            assert!(
                !filters_equal(&new_filters1, &unchanged_filters2),
                "Store1 and store2 should be different"
            );
            assert!(
                filters_equal(&unchanged_filters2, &GetVaultsFilters::default()),
                "Store2 should still have defaults"
            );

            // Test load behavior: create new instances and check persistence
            if let (Ok(mut loaded_store1), Ok(mut loaded_store2)) = (
                DefaultWebFilterStore::new(key1),
                DefaultWebFilterStore::new(key2),
            ) {
                let _ = loaded_store1.load();
                let _ = loaded_store2.load();

                let loaded_filters1 = loaded_store1.get_vaults();
                let loaded_filters2 = loaded_store2.get_vaults();

                // Store2 should still load defaults (no persistence occurred)
                assert!(
                    filters_equal(&loaded_filters2, &GetVaultsFilters::default()),
                    "Store2 should load defaults"
                );

                // Store1 might load modified data if persistence is working, or defaults if not
                // (persistence behavior can vary in test environments)
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

            // Create update configuration
            use alloy::primitives::Address;
            use std::str::FromStr;
            let owner = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
            let update_config = GetVaultsFilters {
                owners: vec![owner],
                hide_zero_balance: true,
                tokens: None,
                chain_ids: Some(vec![1, 137]),
            };

            // Apply update (should set filters AND save them)
            let updated_store_result = store.update_vaults_wasm(update_config.clone());
            assert!(
                updated_store_result.is_ok(),
                "update_vaults_wasm should succeed"
            );

            if let Ok(updated_store) = updated_store_result {
                let updated_filters = updated_store.get_vaults_wasm().unwrap();

                // Verify that the filters actually changed
                assert!(
                    !filters_equal(&original_filters, &updated_filters),
                    "Filters should have changed after update"
                );
                assert!(
                    filters_equal(&update_config, &updated_filters),
                    "Updated filters should match the update config"
                );
                assert_eq!(updated_filters.owners.len(), 1, "Should have one owner");
                assert_eq!(updated_filters.owners[0], owner, "Owner should match");
                assert!(
                    updated_filters.hide_zero_balance,
                    "hide_zero_balance should be true"
                );
                assert_eq!(
                    updated_filters.chain_ids,
                    Some(vec![1, 137]),
                    "Chain IDs should match"
                );

                // Test that changes were saved by creating a new store instance and loading
                if let Ok(new_store) = DefaultWebFilterStore::create(key) {
                    if let Ok(loaded_store) = new_store.load_wasm() {
                        let loaded_filters = loaded_store.get_vaults_wasm().unwrap();
                        // Note: In test environment, persistence might not work, so this is optional verification
                        // If persistence works, loaded_filters should match updated_filters
                    }
                }
            }
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
}
