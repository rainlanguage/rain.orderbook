use serde::{Deserialize, Serialize};

use crate::raindex_client::filters::{
    errors::{FilterError, PersistentFilterStoreError},
    vaults_builder::VaultsFilterBuilder,
    vaults_filter::GetVaultsFilters,
};

/// Builder trait for constructing filter builder.
/// Must implement `build` method to return the final filter type
pub trait FilterBuilder {
    type Output;
    fn build(self) -> Self::Output;
}

/// Filter trait for converting filters to URL parameters and vice versa.
/// Must implement `to_url_params` to convert the filter to URL parameters
/// and `from_url_params` to create a filter from URL parameters.
pub trait Filter: Serialize + for<'de> Deserialize<'de> {
    fn to_url_params(&self) -> String;
    fn from_url_params(params: String) -> Result<Self, FilterError>;
}

/// FilterStore trait for managing filters
pub trait FilterStore {
    fn set_vaults_filters(&mut self, filters: GetVaultsFilters);
    fn update_vaults_filters<F>(&mut self, update_fn: F)
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder;
}

/// PersistentFilterStore trait for managing filters with persistence
pub trait PersistentFilterStore: FilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError>;
    fn save(&self) -> Result<(), PersistentFilterStoreError>;
}
