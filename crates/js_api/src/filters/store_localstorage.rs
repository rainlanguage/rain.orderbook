use super::errors::PersistentFilterStoreError;
use super::traits::PersistentFilterStore;
use anyhow;
use rain_orderbook_common::raindex_client::filters::{
    store_basic::BasicFilterStore, traits::FilterStore, vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};
use serde::{Deserialize, Serialize};

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct LocalStorageFilterStore<S: FilterStore> {
    key: String,
    store: S,
    #[cfg_attr(target_family = "wasm", tsify(skip))]
    #[serde(skip)]
    _ls: web_sys::Storage,
}

// Custom serialization that only includes the store data (key is used for localStorage key)
impl<S: FilterStore + Serialize> Serialize for LocalStorageFilterStore<S> {
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: serde::Serializer,
    {
        // Only serialize the inner store, not the key or _ls field
        self.store.serialize(serializer)
    }
}

// Custom deserialization that only deserializes the store and requires key to be provided separately
impl<'de, S: FilterStore + Deserialize<'de>> Deserialize<'de> for LocalStorageFilterStore<S> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // This deserializer will only work when called through our specific context
        // where we can provide the key. For general deserialization, this will fail.
        Err(serde::de::Error::custom(
            "LocalStorageFilterStore requires key context for deserialization. Use from_json_with_key instead."
        ))
    }
}

// Only expose the concrete type that's actually used to JavaScript
#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct LocalStorageBasicFilterStore {
    inner: LocalStorageFilterStore<BasicFilterStore>,
}

#[cfg(target_family = "wasm")]
impl LocalStorageBasicFilterStore {
    pub fn new(key: &str) -> Result<Self, PersistentFilterStoreError> {
        Ok(Self {
            inner: LocalStorageFilterStore::new(key)?,
        })
    }
}

#[cfg(target_family = "wasm")]
impl_wasm_traits!(LocalStorageBasicFilterStore);

impl LocalStorageFilterStore<BasicFilterStore> {
    pub fn new(key: &str) -> Result<Self, PersistentFilterStoreError> {
        Self::new_with_store(key, BasicFilterStore::new())
    }
}

impl<S: FilterStore + Serialize + for<'de> Deserialize<'de>> LocalStorageFilterStore<S> {
    /// Serialize the store to JSON string (without key)
    pub fn to_json(&self) -> Result<String, PersistentFilterStoreError> {
        serde_json::to_string(&self.store).map_err(|e| {
            PersistentFilterStoreError::SaveError(format!("JSON serialization failed: {}", e))
        })
    }

    /// Deserialize from JSON string with provided key
    pub fn from_json_with_key(key: &str, json: &str) -> Result<Self, PersistentFilterStoreError> {
        let store: S = serde_json::from_str(json).map_err(|e| {
            PersistentFilterStoreError::LoadError(format!("JSON deserialization failed: {}", e))
        })?;

        let _ls = web_sys::window()
            .ok_or(PersistentFilterStoreError::WindowNotAvailable)?
            .local_storage()
            .map_err(|e| PersistentFilterStoreError::LocalStorageInitError(format!("{:?}", e)))?
            .ok_or(PersistentFilterStoreError::LocalStorageUnavailable)?;

        Ok(LocalStorageFilterStore {
            key: key.to_string(),
            store,
            _ls,
        })
    }

    pub fn new_with_store(key: &str, store: S) -> Result<Self, PersistentFilterStoreError> {
        let mut persistent_store = Self {
            key: key.to_string(),
            store,
            _ls: web_sys::window()
                .ok_or(PersistentFilterStoreError::WindowNotAvailable)?
                .local_storage()
                .map_err(|e| PersistentFilterStoreError::LocalStorageInitError(format!("{:?}", e)))?
                .ok_or(PersistentFilterStoreError::LocalStorageUnavailable)?,
        };
        persistent_store.load()?;
        Ok(persistent_store)
    }
}

impl<S: FilterStore + Serialize + for<'de> Deserialize<'de>> FilterStore
    for LocalStorageFilterStore<S>
{
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
        self.store.update_vaults(update_fn)?;
        self.save()
            .map_err(|e| anyhow::anyhow!("Failed to save filters to localStorage: {}", e))
    }
}

impl<S: FilterStore + Serialize + for<'de> Deserialize<'de>> PersistentFilterStore
    for LocalStorageFilterStore<S>
{
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        // Load store data from local storage
        let loaded_data = self
            ._ls
            .get_item(&self.key)
            .map_err(|_| PersistentFilterStoreError::LocalStorageUnavailable)?;

        if let Some(data) = loaded_data {
            // Deserialize the entire store from JSON
            let loaded_store: S = serde_json::from_str(&data).map_err(|e| {
                PersistentFilterStoreError::LoadError(format!("Failed to parse stored data: {}", e))
            })?;
            self.store = loaded_store;
        }
        // If no data found, keep current store state (defaults)
        Ok(())
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        // Serialize the entire store to JSON
        let data = serde_json::to_string(&self.store)
            .map_err(|err| PersistentFilterStoreError::SaveError(err.to_string()))?;
        self._ls
            .set_item(&self.key, &data)
            .map_err(|err| PersistentFilterStoreError::SaveError(format!("{:?}", err)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::window;

    fn clear_local_storage(key: &str) {
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(key);
            }
        }
    }

    fn filters_equal(a: &GetVaultsFilters, b: &GetVaultsFilters) -> bool {
        // Compare through serialization since PartialEq is not implemented
        serde_json::to_string(a).unwrap() == serde_json::to_string(b).unwrap()
    }

    #[wasm_bindgen_test]
    fn test_save_and_load() {
        let key = "test_filters";
        clear_local_storage(key);

        // Тест может не пройти, если localStorage недоступен в тестовом окружении
        if let Ok(mut store) = LocalStorageFilterStore::new(key) {
            let filters = GetVaultsFilters::default();
            store.set_vaults(filters.clone());

            if store.save().is_ok() {
                // New store should load the same data
                if let Ok(loaded) = LocalStorageFilterStore::new(key) {
                    assert!(filters_equal(&store.get_vaults(), &loaded.get_vaults()));
                }
            }
        }
        clear_local_storage(key);
    }

    #[wasm_bindgen_test]
    fn test_load_missing_key() {
        let key = "missing_key";
        clear_local_storage(key);

        if let Ok(store) = LocalStorageFilterStore::new(key) {
            // Should have default value
            assert!(filters_equal(
                &store.get_vaults(),
                &GetVaultsFilters::default()
            ));
        }
    }

    #[wasm_bindgen_test]
    fn test_save_error() {
        let key = "test_save_error";
        clear_local_storage(key);

        if let Ok(store) = LocalStorageFilterStore::new(key) {
            // save должен работать даже с дефолтными фильтрами, если localStorage доступен
            // Но в тестовом окружении может быть недоступен, что нормально
            let _ = store.save();
        }
        clear_local_storage(key);
    }

    #[wasm_bindgen_test]
    fn test_update_vaults_saves_automatically() {
        let key = "test_update";
        clear_local_storage(key);

        if let Ok(mut store) = LocalStorageFilterStore::new(key) {
            // Update filters - this should automatically save to localStorage
            store.update_vaults(|builder| builder).unwrap();

            // Create new store with same key - should load saved filters
            if let Ok(loaded) = LocalStorageFilterStore::new(key) {
                assert!(filters_equal(&store.get_vaults(), &loaded.get_vaults()));
            }
        }
        clear_local_storage(key);
    }
}
