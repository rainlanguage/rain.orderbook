use serde::{Deserialize, Serialize};

use crate::raindex_client::filters::{
    vaults_builder::VaultsFilterBuilder, vaults_filter::GetVaultsFilters,
};

/// Builder trait for constructing filter builder.
/// Must implement `build` method to return the final filter type
pub trait FilterBuilder {
    type Output;
    fn build(self) -> Self::Output;
}

/// Filter trait should implement basic traits
pub trait Filter: Serialize + for<'de> Deserialize<'de> {}

/// FilterStore trait for managing filters
pub trait FilterStore {
    fn set_vaults(&mut self, filters: GetVaultsFilters);
    fn update_vaults<F>(&mut self, update_fn: F)
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder;
}
