use crate::raindex_client::filters::{
    errors::PersistentFilterStoreError,
    store_basic::BasicFilterStore,
    traits::{FilterStore, PersistentFilterStore},
    vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};

pub struct LocalStorageFilterStore {
    key: String,
    store: BasicFilterStore,
}

impl LocalStorageFilterStore {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            store: BasicFilterStore::new(),
        }
    }
}

impl FilterStore for LocalStorageFilterStore {
    fn set_vaults_filters(&mut self, filters: GetVaultsFilters) {
        self.store.set_vaults_filters(filters);
    }

    fn update_vaults_filters<F>(&mut self, update_fn: F)
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder,
    {
        self.store.update_vaults_filters(update_fn);
    }
}

impl PersistentFilterStore for LocalStorageFilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError> {
        // Load filters from local storage (not implemented here)
        todo!("Implement loading from local storage");
    }

    fn save(&self) -> Result<(), PersistentFilterStoreError> {
        // Save filters to local storage (not implemented here)
        todo!("Implement saving to local storage");
    }
}
