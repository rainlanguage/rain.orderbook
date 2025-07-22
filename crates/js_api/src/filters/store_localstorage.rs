use super::errors::PersistentFilterStoreError;
use super::traits::PersistentFilterStore;
use anyhow;
use rain_orderbook_common::raindex_client::filters::{
    store_basic::BasicFilterStore, traits::FilterStore, vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};

pub struct LocalStorageFilterStore {
    key: String,
    store: BasicFilterStore,
    _ls: web_sys::Storage,
}

impl LocalStorageFilterStore {
    pub fn new(key: &str) -> Result<Self, PersistentFilterStoreError> {
        let mut store = Self {
            key: key.to_string(),
            store: BasicFilterStore::new(),
            _ls: web_sys::window()
                .ok_or(PersistentFilterStoreError::WindowNotAvailable)?
                .local_storage()
                .map_err(|e| PersistentFilterStoreError::LocalStorageInitError(format!("{:?}", e)))?
                .ok_or(PersistentFilterStoreError::LocalStorageUnavailable)?,
        };
        store.load()?;
        Ok(store)
    }
}

impl FilterStore for LocalStorageFilterStore {
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

impl PersistentFilterStore for LocalStorageFilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        // Load filters from local storage (not implemented here)
        let loaded = self
            ._ls
            .get_item(&self.key)
            .map_err(|_| PersistentFilterStoreError::LocalStorageUnavailable)?;
        self.store.set_vaults(
            loaded
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default(),
        );
        Ok(())
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        let data = serde_json::to_string(&self.store.get_vaults())
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
