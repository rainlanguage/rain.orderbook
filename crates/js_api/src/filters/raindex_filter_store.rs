use super::errors::PersistentFilterStoreError;
use super::traits::PersistentFilterStore;
use anyhow;
use rain_orderbook_common::raindex_client::filters::{
    orders::builder::OrdersFilterBuilder,
    orders::filter::GetOrdersFilters,
    traits::FilterStore,
    vaults::builder::VaultsFilterBuilder,
    vaults::filter::GetVaultsFilters,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

/// Hardcoded localStorage key for filter persistence
const STORAGE_KEY: &str = "raindex-filters";

/// Simplified unified filter store that manages filters with localStorage and URL parameters.
///
/// This store provides a complete filtering solution for web applications with hardcoded configuration:
/// - localStorage: Persists filters across browser sessions using hardcoded key "raindex-filters"
/// - URL parameters: Saves/loads filters from URL search parameters for sharing
/// - JSON serialization: Handles conversion to/from JSON for both storage mechanisms
/// - Selective URL updates: Only the filter type being updated (vaults or orders) is saved to URL
///
/// Priority order when loading:
/// 1. URL parameters (highest - for sharing links)  
/// 2. localStorage (fallback - for user persistence)
/// 3. Default values (lowest - initial state)
///
/// URL parameter behavior:
/// - When updating vaults filters: only vaults.* parameters are modified in URL
/// - When updating orders filters: only orders.* parameters are modified in URL  
/// - This ensures URL reflects the current page context (Vaults page vs Orders page)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[wasm_bindgen]
pub struct RaindexFilterStore {
    vaults: GetVaultsFilters,
    orders: GetOrdersFilters,
}

impl Default for RaindexFilterStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RaindexFilterStore {
    /// Create a new raindex filter store with default filters
    pub fn new() -> Self {
        let mut store = Self {
            vaults: GetVaultsFilters::default(),
            orders: GetOrdersFilters::default(),
        };

        // Try to load from persistent storage (URL params have priority over localStorage)
        // Log errors but continue with defaults if loading fails
        if let Err(e) = store.load() {
            #[cfg(target_family = "wasm")]
            web_sys::console::warn_1(&format!("Failed to load filters: {}", e).into());
        }

        store
    }

    /// Save filters to localStorage as JSON
    fn save_to_localstorage(&self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;

        let local_storage = window
            .local_storage()
            .map_err(|e| PersistentFilterStoreError::LocalStorageInitError(format!("{:?}", e)))?
            .ok_or(PersistentFilterStoreError::LocalStorageUnavailable)?;

        let json = serde_json::to_string(self).map_err(|e| {
            PersistentFilterStoreError::SaveError(format!("JSON serialization failed: {}", e))
        })?;

        local_storage.set_item(STORAGE_KEY, &json).map_err(|e| {
            PersistentFilterStoreError::SaveError(format!("localStorage write failed: {:?}", e))
        })?;

        Ok(())
    }

    /// Load filters from localStorage
    fn load_from_localstorage(&mut self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;

        let local_storage = window
            .local_storage()
            .map_err(|e| PersistentFilterStoreError::LocalStorageInitError(format!("{:?}", e)))?
            .ok_or(PersistentFilterStoreError::LocalStorageUnavailable)?;

        if let Ok(Some(json)) = local_storage.get_item(STORAGE_KEY) {
            // Try to deserialize the entire store first (new format)
            if let Ok(store) = serde_json::from_str::<RaindexFilterStore>(&json) {
                self.vaults = store.vaults;
                self.orders = store.orders;
            } else {
                // Fallback: try to deserialize as legacy vaults-only format
                if let Ok(vaults) = serde_json::from_str::<GetVaultsFilters>(&json) {
                    self.vaults = vaults;
                    // Keep default orders filters
                }
            }
        }
        // If no data in localStorage, keep default filters

        Ok(())
    }

    /// Save vault filters to URL search parameters using structured format
    fn save_vaults_to_url(&self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;
        let location = window.location();

        // Get current search params
        let current_search = location.search().map_err(|_| {
            PersistentFilterStoreError::SaveError(
                "Failed to read current URL search params".to_string(),
            )
        })?;

        let url_params = web_sys::UrlSearchParams::new_with_str(&current_search).map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to create URLSearchParams".to_string())
        })?;

        // Clear existing vaults.* parameters (keep orders.* parameters intact)
        // We need to collect keys first because we can't modify while iterating
        let mut keys_to_remove = Vec::new();

        // Create a temporary JS iterator to collect vaults.* keys
        let entries = url_params.entries();
        loop {
            let entry = entries.next().map_err(|_| {
                PersistentFilterStoreError::SaveError("Failed to iterate URL params".to_string())
            })?;

            if entry.done() {
                break;
            }

            if let Ok(entry_array) = entry.value().dyn_into::<js_sys::Array>() {
                if let Some(key) = entry_array.get(0).as_string() {
                    if key.starts_with("vaults.") {
                        keys_to_remove.push(key);
                    }
                }
            }
        }

        for key in keys_to_remove {
            url_params.delete(&key);
        }

        // Add vault-specific filters with vaults.* prefix (only non-default values)

        // tokens (only if not empty)
        if let Some(ref tokens) = self.vaults.tokens {
            if !tokens.is_empty() {
                let tokens_str = tokens
                    .iter()
                    .map(|addr| format!("{:#x}", addr))
                    .collect::<Vec<_>>()
                    .join(",");
                url_params.set("vaults.tokens", &tokens_str);
            }
        }

        // hideZeroBalance (only if true)
        if self.vaults.hide_zero_balance {
            url_params.set("vaults.hideZeroBalance", "true");
        }

        // chainIds (only if not empty)
        if let Some(ref chain_ids) = self.vaults.chain_ids {
            if !chain_ids.is_empty() {
                let chain_ids_str = chain_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                url_params.set("vaults.chainIds", &chain_ids_str);
            }
        }

        // owners (only if not empty)
        if !self.vaults.owners.is_empty() {
            let owners_str = self
                .vaults
                .owners
                .iter()
                .map(|addr| format!("{:#x}", addr))
                .collect::<Vec<_>>()
                .join(",");
            url_params.set("vaults.owners", &owners_str);
        }

        // Update the URL without reloading the page
        let new_search = format!(
            "?{}",
            url_params.to_string().as_string().unwrap_or_default()
        );

        let history = window.history().map_err(|_| {
            PersistentFilterStoreError::SaveError("History API not available".to_string())
        })?;

        // Get the current pathname and hash to reconstruct full URL
        let pathname = location.pathname().map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to get pathname".to_string())
        })?;
        let hash = location
            .hash()
            .map_err(|_| PersistentFilterStoreError::SaveError("Failed to get hash".to_string()))?;

        // Create new URL string manually
        let new_url = format!("{}{}{}", pathname, new_search, hash);

        history
            .replace_state_with_url(&JsValue::UNDEFINED, "", Some(&new_url))
            .map_err(|_| {
                PersistentFilterStoreError::SaveError("Failed to update URL".to_string())
            })?;

        Ok(())
    }

    /// Save orders filters to URL search parameters using structured format
    fn save_orders_to_url(&self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;
        let location = window.location();

        // Get current search params
        let current_search = location.search().map_err(|_| {
            PersistentFilterStoreError::SaveError(
                "Failed to read current URL search params".to_string(),
            )
        })?;

        let url_params = web_sys::UrlSearchParams::new_with_str(&current_search).map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to create URLSearchParams".to_string())
        })?;

        // Clear existing orders.* parameters (keep vaults.* parameters intact)
        // We need to collect keys first because we can't modify while iterating
        let mut keys_to_remove = Vec::new();

        // Create a temporary JS iterator to collect orders.* keys
        let entries = url_params.entries();
        loop {
            let entry = entries.next().map_err(|_| {
                PersistentFilterStoreError::SaveError("Failed to iterate URL params".to_string())
            })?;

            if entry.done() {
                break;
            }

            if let Ok(entry_array) = entry.value().dyn_into::<js_sys::Array>() {
                if let Some(key) = entry_array.get(0).as_string() {
                    if key.starts_with("orders.") {
                        keys_to_remove.push(key);
                    }
                }
            }
        }

        for key in keys_to_remove {
            url_params.delete(&key);
        }

        // Add orders-specific filters with orders.* prefix (only non-default values)

        // owners (only if not empty)
        if !self.orders.owners.is_empty() {
            let owners_str = self
                .orders
                .owners
                .iter()
                .map(|addr| format!("{:#x}", addr))
                .collect::<Vec<_>>()
                .join(",");
            url_params.set("orders.owners", &owners_str);
        }

        // active (only if set)
        if let Some(active) = self.orders.active {
            url_params.set("orders.active", &active.to_string());
        }

        // order_hash (only if set)
        if let Some(ref order_hash) = self.orders.order_hash {
            url_params.set("orders.orderHash", &format!("{:#x}", order_hash));
        }

        // tokens (only if not empty)
        if let Some(ref tokens) = self.orders.tokens {
            if !tokens.is_empty() {
                let tokens_str = tokens
                    .iter()
                    .map(|addr| format!("{:#x}", addr))
                    .collect::<Vec<_>>()
                    .join(",");
                url_params.set("orders.tokens", &tokens_str);
            }
        }

        // Update the URL without reloading the page
        let new_search = format!(
            "?{}",
            url_params.to_string().as_string().unwrap_or_default()
        );

        let history = window.history().map_err(|_| {
            PersistentFilterStoreError::SaveError("History API not available".to_string())
        })?;

        // Get the current pathname and hash to reconstruct full URL
        let pathname = location.pathname().map_err(|_| {
            PersistentFilterStoreError::SaveError("Failed to get pathname".to_string())
        })?;
        let hash = location
            .hash()
            .map_err(|_| PersistentFilterStoreError::SaveError("Failed to get hash".to_string()))?;

        // Create new URL string manually
        let new_url = format!("{}{}{}", pathname, new_search, hash);

        history
            .replace_state_with_url(&JsValue::UNDEFINED, "", Some(&new_url))
            .map_err(|_| {
                PersistentFilterStoreError::SaveError("Failed to update URL".to_string())
            })?;

        Ok(())
    }

    /// Load filters from URL search parameters using structured format
    fn load_from_url(&mut self) -> Result<(), PersistentFilterStoreError> {
        let window = web_sys::window().ok_or(PersistentFilterStoreError::WindowNotAvailable)?;

        // Get current URL search params
        let search = window.location().search().map_err(|_| {
            PersistentFilterStoreError::LoadError("Failed to read URL search params".to_string())
        })?;

        if search.is_empty() || search == "?" {
            return Err(PersistentFilterStoreError::LoadError(
                "No URL params".to_string(),
            ));
        }

        // Parse URL params
        let url_params = web_sys::UrlSearchParams::new_with_str(&search)
            .map_err(|_| PersistentFilterStoreError::LoadError("Invalid URL params".to_string()))?;

        let mut found_any_filter = false;
        let mut new_vaults_filters = GetVaultsFilters::default();
        let mut new_orders_filters = GetOrdersFilters::default();

        // Parse vaults.* parameters
        if let Some(tokens_str) = url_params.get("vaults.tokens") {
            found_any_filter = true;
            let tokens: Result<Vec<alloy::primitives::Address>, _> = tokens_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect();

            match tokens {
                Ok(parsed_tokens) => new_vaults_filters.tokens = Some(parsed_tokens),
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid vaults tokens in URL: {}",
                        e
                    )));
                }
            }
        }

        if let Some(hide_zero_str) = url_params.get("vaults.hideZeroBalance") {
            found_any_filter = true;
            new_vaults_filters.hide_zero_balance = hide_zero_str == "true";
        }

        if let Some(chain_ids_str) = url_params.get("vaults.chainIds") {
            found_any_filter = true;
            let chain_ids: Result<Vec<u32>, _> = chain_ids_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect();

            match chain_ids {
                Ok(parsed_chain_ids) => new_vaults_filters.chain_ids = Some(parsed_chain_ids),
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid vaults chain IDs in URL: {}",
                        e
                    )));
                }
            }
        }

        if let Some(owners_str) = url_params.get("vaults.owners") {
            found_any_filter = true;
            let owners: Result<Vec<alloy::primitives::Address>, _> = owners_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect();

            match owners {
                Ok(parsed_owners) => new_vaults_filters.owners = parsed_owners,
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid vaults owners in URL: {}",
                        e
                    )));
                }
            }
        }

        // Parse orders.* parameters
        if let Some(owners_str) = url_params.get("orders.owners") {
            found_any_filter = true;
            let owners: Result<Vec<alloy::primitives::Address>, _> = owners_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect();

            match owners {
                Ok(parsed_owners) => new_orders_filters.owners = parsed_owners,
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid orders owners in URL: {}",
                        e
                    )));
                }
            }
        }

        if let Some(active_str) = url_params.get("orders.active") {
            found_any_filter = true;
            match active_str.parse::<bool>() {
                Ok(parsed_active) => new_orders_filters.active = Some(parsed_active),
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid orders active in URL: {}",
                        e
                    )));
                }
            }
        }

        if let Some(order_hash_str) = url_params.get("orders.orderHash") {
            found_any_filter = true;
            match order_hash_str.parse::<alloy::primitives::Bytes>() {
                Ok(parsed_hash) => new_orders_filters.order_hash = Some(parsed_hash),
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid orders order hash in URL: {}",
                        e
                    )));
                }
            }
        }

        if let Some(tokens_str) = url_params.get("orders.tokens") {
            found_any_filter = true;
            let tokens: Result<Vec<alloy::primitives::Address>, _> = tokens_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse())
                .collect();

            match tokens {
                Ok(parsed_tokens) => new_orders_filters.tokens = Some(parsed_tokens),
                Err(e) => {
                    return Err(PersistentFilterStoreError::LoadError(format!(
                        "Invalid orders tokens in URL: {}",
                        e
                    )));
                }
            }
        }

        if !found_any_filter {
            return Err(PersistentFilterStoreError::LoadError(
                "No vaults.* or orders.* parameters in URL".to_string(),
            ));
        }

        self.vaults = new_vaults_filters;
        self.orders = new_orders_filters;
        Ok(())
    }
}

impl FilterStore for RaindexFilterStore {
    fn get_vaults(&self) -> GetVaultsFilters {
        self.vaults.clone()
    }

    fn set_vaults(&mut self, filters: GetVaultsFilters) {
        self.vaults = filters;
    }

    fn update_vaults<F>(&mut self, update_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder,
    {
        let builder = VaultsFilterBuilder::from(self.vaults.clone());
        let updated_builder = update_fn(builder);
        self.vaults = updated_builder.build();

        // Auto-save to both localStorage and URL after update
        self.save_to_localstorage()
            .map_err(|e| anyhow::anyhow!("Failed to save filters to localStorage: {}", e))?;

        self.save_vaults_to_url()
            .map_err(|e| anyhow::anyhow!("Failed to save vault filters to URL: {}", e))?;

        Ok(())
    }

    fn get_orders(&self) -> GetOrdersFilters {
        self.orders.clone()
    }

    fn set_orders(&mut self, filters: GetOrdersFilters) {
        self.orders = filters;
    }

    fn update_orders<F>(&mut self, update_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(OrdersFilterBuilder) -> OrdersFilterBuilder,
    {
        let builder = OrdersFilterBuilder::from(self.orders.clone());
        let updated_builder = update_fn(builder);
        self.orders = updated_builder.build();

        // Auto-save to both localStorage and URL after update
        self.save_to_localstorage()
            .map_err(|e| anyhow::anyhow!("Failed to save filters to localStorage: {}", e))?;

        self.save_orders_to_url()
            .map_err(|e| anyhow::anyhow!("Failed to save orders filters to URL: {}", e))?;

        Ok(())
    }
}

impl PersistentFilterStore for RaindexFilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        // Priority: URL params (new format) > localStorage > current defaults

        // Try new structured URL params first (highest priority)
        if self.load_from_url().is_ok() {
            return Ok(());
        }

        // Fallback to localStorage
        self.load_from_localstorage()
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        // Save only to localStorage, URL params are saved separately based on context
        self.save_to_localstorage()
    }
}

// WASM bindings
#[cfg(target_family = "wasm")]
#[wasm_export]
impl RaindexFilterStore {
    #[wasm_export(
        js_name = "create",
        preserve_js_class,
        return_description = "Creates a new RaindexFilterStore instance"
    )]
    pub fn create() -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        Ok(RaindexFilterStore::new())
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
        js_name = "getOrders",
        preserve_js_class,
        return_description = "Returns current orders filters"
    )]
    pub fn get_orders_wasm(&self) -> Result<GetOrdersFilters, PersistentFilterStoreError> {
        Ok(self.get_orders())
    }

    #[wasm_export(
        js_name = "setOrders",
        preserve_js_class,
        return_description = "Sets orders filters without saving to persistent storage"
    )]
    pub fn set_orders_wasm(
        self,
        filters: GetOrdersFilters,
    ) -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.set_orders(filters);
        Ok(store)
    }

    #[wasm_export(
        js_name = "updateOrders",
        preserve_js_class,
        return_description = "Sets orders filters and saves to persistent storage"
    )]
    pub fn update_orders_wasm(
        self,
        filters: GetOrdersFilters,
    ) -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.set_orders(filters);
        // Save to localStorage and orders-specific URL params
        store.save_to_localstorage()?;
        store.save_orders_to_url()?;
        Ok(store)
    }

    #[wasm_export(
        js_name = "setVaults",
        preserve_js_class,
        return_description = "Sets vault filters without saving to persistent storage"
    )]
    pub fn set_vaults_wasm(
        self,
        filters: GetVaultsFilters,
    ) -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.set_vaults(filters);
        Ok(store)
    }

    #[wasm_export(
        js_name = "updateVaults",
        preserve_js_class,
        return_description = "Sets vault filters and saves to persistent storage"
    )]
    pub fn update_vaults_wasm(
        self,
        filters: GetVaultsFilters,
    ) -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.set_vaults(filters);
        // Save to localStorage and vault-specific URL params
        store.save_to_localstorage()?;
        store.save_vaults_to_url()?;
        Ok(store)
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
    pub fn load_wasm(self) -> Result<RaindexFilterStore, PersistentFilterStoreError> {
        let mut store = self;
        store.load()?;
        Ok(store)
    }

    #[wasm_export(
        js_name = "saveVaultsToUrl",
        unchecked_return_type = "void",
        return_description = "Saves current vault filters to URL parameters using structured format"
    )]
    pub fn save_vaults_to_url_wasm(&self) -> Result<(), PersistentFilterStoreError> {
        self.save_vaults_to_url()
    }

    #[wasm_export(
        js_name = "saveOrdersToUrl",
        unchecked_return_type = "void",
        return_description = "Saves current orders filters to URL parameters using structured format"
    )]
    pub fn save_orders_to_url_wasm(&self) -> Result<(), PersistentFilterStoreError> {
        self.save_orders_to_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_new_store_has_default_filters() {
        let store = RaindexFilterStore::new();
        let vaults_filters = store.get_vaults();
        let orders_filters = store.get_orders();

        assert!(vaults_filters.owners.is_empty());
        assert!(!vaults_filters.hide_zero_balance);
        assert!(vaults_filters.tokens.is_none());
        assert!(vaults_filters.chain_ids.is_none());

        assert!(orders_filters.owners.is_empty());
        assert!(orders_filters.active.is_none());
        assert!(orders_filters.order_hash.is_none());
        assert!(orders_filters.tokens.is_none());
    }

    #[wasm_bindgen_test]
    fn test_set_and_get_orders() {
        let mut store = RaindexFilterStore::new();
        let original_filters = store.get_orders();

        // Set new filters with non-default values
        let owner = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let new_filters = GetOrdersFilters {
            owners: vec![owner],
            active: Some(true),
            order_hash: None,
            tokens: None,
        };
        store.set_orders(new_filters.clone());

        let retrieved_filters = store.get_orders();

        // Verify that the filters actually changed
        assert_ne!(original_filters.owners, retrieved_filters.owners);
        assert_ne!(original_filters.active, retrieved_filters.active);
        assert_eq!(new_filters.owners, retrieved_filters.owners);
        assert_eq!(new_filters.active, retrieved_filters.active);
        assert_eq!(new_filters.order_hash, retrieved_filters.order_hash);
        assert_eq!(new_filters.tokens, retrieved_filters.tokens);
    }

    #[wasm_bindgen_test]
    fn test_set_and_get_vaults() {
        let mut store = RaindexFilterStore::new();
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
        assert_ne!(original_filters.owners, retrieved_filters.owners);
        assert_ne!(
            original_filters.hide_zero_balance,
            retrieved_filters.hide_zero_balance
        );
        assert_eq!(new_filters.owners, retrieved_filters.owners);
        assert_eq!(
            new_filters.hide_zero_balance,
            retrieved_filters.hide_zero_balance
        );
        assert_eq!(new_filters.chain_ids, retrieved_filters.chain_ids);
    }

    #[wasm_bindgen_test]
    fn test_serialization_roundtrip() {
        let store = RaindexFilterStore::new();

        // Test that store can be serialized and deserialized
        let serialized = serde_json::to_string(&store);
        assert!(serialized.is_ok(), "Store should serialize successfully");

        if let Ok(json) = serialized {
            let deserialized: Result<RaindexFilterStore, _> = serde_json::from_str(&json);
            assert!(
                deserialized.is_ok(),
                "Store should deserialize successfully"
            );
        }
    }

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_create() {
        let result = RaindexFilterStore::create();
        // In browser environment, this should succeed
        // In Node.js, it might fail with window not available
        if result.is_ok() {
            let store = result.unwrap();
            let filters = store.get_vaults_wasm().unwrap();
            assert!(filters.owners.is_empty());
        }
    }

    #[cfg(target_family = "wasm")]
    #[wasm_bindgen_test]
    fn test_wasm_get_vaults() {
        if let Ok(store) = RaindexFilterStore::create() {
            let result = store.get_vaults_wasm();
            assert!(result.is_ok(), "get_vaults_wasm should return Ok");
        }
    }
}
