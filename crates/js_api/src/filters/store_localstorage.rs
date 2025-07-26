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
