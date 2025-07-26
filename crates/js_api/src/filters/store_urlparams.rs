use super::errors::PersistentFilterStoreError;
use super::store_localstorage::LocalStorageFilterStore;
use super::traits::PersistentFilterStore;
use anyhow;
use rain_orderbook_common::raindex_client::filters::{
    store_basic::BasicFilterStore, traits::FilterStore, vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

/// URLParams filter store that manages filters in URL search parameters
/// with fallback to an underlying persistent store (typically LocalStorageFilterStore)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct URLParamsFilterStore<S: FilterStore> {
    store: S,
}

impl URLParamsFilterStore<LocalStorageFilterStore<BasicFilterStore>> {
    /// Create new URLParams store with LocalStorage as underlying store
    pub fn new(key: &str) -> Result<Self, PersistentFilterStoreError> {
        let local_storage_store = LocalStorageFilterStore::new(key)?;
        Self::new_with_store(local_storage_store)
    }
}

impl<S: FilterStore> URLParamsFilterStore<S> {
    /// Create new URLParams store with custom underlying store
    pub fn new_with_store(store: S) -> Result<Self, PersistentFilterStoreError> {
        let mut url_store = Self { store };
        // Load from URL params first (priority), then fallback to underlying store
        url_store.load()?;
        Ok(url_store)
    }
}

impl<S: FilterStore> FilterStore for URLParamsFilterStore<S> {
    fn get_vaults(&self) -> GetVaultsFilters {
        self.store.get_vaults()
    }

    fn set_vaults(&mut self, filters: GetVaultsFilters) {
        self.store.set_vaults(filters);
    }

    fn update_vaults<F>(&mut self, update_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder,
    {
        // Update underlying store first
        self.store.update_vaults(update_fn)?;
        // Then save to URL params
        self.save()
            .map_err(|e| anyhow::anyhow!("Failed to save filters to URL params: {}", e))
    }
}

impl<S: FilterStore> PersistentFilterStore for URLParamsFilterStore<S> {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        // Try to load from URL params first
        if let Ok(url_filters) = self.load_from_url() {
            // URL params have priority - overwrite underlying store data
            self.store.set_vaults(url_filters);
        }
        // If URL params are empty or invalid, underlying store data is kept
        Ok(())
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        // Save to URL params
        self.save_to_url()
        // Note: underlying store is already updated in update_vaults()
    }
}

impl<S: FilterStore> URLParamsFilterStore<S> {
    /// Load filters from URL search parameters
    fn load_from_url(&self) -> Result<GetVaultsFilters, PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;

        // Get current URL search params
        let search = window.location().search().map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to read URL search params".to_string())
        })?;

        if search.is_empty() || search == "?" {
            return Err(PersistentFilterStoreError::SaveError(
                "No URL params".to_string(),
            ));
        }

        // Parse URL params
        let url_params = web_sys::UrlSearchParams::new_with_str(&search)
            .map_err(|_| PersistentFilterStoreError::SaveError("Invalid URL params".to_string()))?;

        // Look for 'filters' parameter with JSON data
        if let Some(filters_json) = url_params.get("filters") {
            serde_json::from_str(&filters_json).map_err(|e| {
                PersistentFilterStoreError::SaveError(format!(
                    "Failed to parse filters from URL: {}",
                    e
                ))
            })
        } else {
            Err(PersistentFilterStoreError::SaveError(
                "No filters in URL params".to_string(),
            ))
        }
    }

    /// Save filters to URL search parameters
    fn save_to_url(&self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;

        // Get current search params to preserve other parameters
        let search = window.location().search().map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to read current URL".to_string())
        })?;
        let url_params = web_sys::UrlSearchParams::new_with_str(&search).map_err(|_| {
            PersistentFilterStoreError::SaveError("Invalid current URL params".to_string())
        })?;

        // Serialize current filters to JSON
        let filters_json = serde_json::to_string(&self.store.get_vaults()).map_err(|e| {
            PersistentFilterStoreError::SaveError(format!("Failed to serialize filters: {}", e))
        })?;

        // Set filters parameter
        url_params.set("filters", &filters_json);

        // Update URL using replaceState (doesn't add history entry)
        let history = window.history().map_err(|_| {
            PersistentFilterStoreError::SaveError("History API not available".to_string())
        })?;

        let pathname = window.location().pathname().map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to read pathname".to_string())
        })?;
        let new_url = format!("{}?{}", pathname, url_params.to_string());

        history
            .replace_state_with_url(&JsValue::NULL, "", Some(&new_url))
            .map_err(|_| {
                PersistentFilterStoreError::SaveError("Failed to update URL".to_string())
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::test_utils::filters_equal;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_url_serialization_format() {
        use rain_orderbook_common::raindex_client::filters::vaults_filter::GetVaultsFilters;

        // Test that serialization produces valid JSON
        let filters = GetVaultsFilters::default();
        let json_result = serde_json::to_string(&filters);
        assert!(
            json_result.is_ok(),
            "Should serialize GetVaultsFilters to JSON"
        );

        // Test that we can deserialize back
        if let Ok(json_str) = json_result {
            let deserialize_result: Result<GetVaultsFilters, _> = serde_json::from_str(&json_str);
            assert!(
                deserialize_result.is_ok(),
                "Should deserialize JSON back to GetVaultsFilters"
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_url_store_preserves_underlying_functionality() {
        let key = "test_url_preserve";

        if let Ok(mut url_store) = URLParamsFilterStore::new(key) {
            // Test that basic FilterStore functionality works through URL wrapper
            let original_filters = url_store.get_vaults();

            // Update and verify it's preserved
            let _ = url_store.update_vaults(|builder| builder);
            let updated_filters = url_store.get_vaults();

            // The filters should be the same since we didn't change anything
            assert!(filters_equal(&original_filters, &updated_filters));
        }
    }
}
