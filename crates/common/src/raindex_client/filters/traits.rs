use anyhow;
use serde::{Deserialize, Serialize};

use crate::raindex_client::filters::vaults::{
    builder::VaultsFilterBuilder, filter::GetVaultsFilters,
};

/// Builder trait for constructing filters.
/// Must implement `build` method to return the final filter type
pub trait FilterBuilder {
    type Output;
    fn build(self) -> Self::Output;
}

/// Filter trait for types that can be persisted and transferred.
/// Requires serialization capabilities for storage
pub trait Filter: Serialize + for<'de> Deserialize<'de> {}

/// FilterStore trait for managing filters
pub trait FilterStore {
    fn get_vaults(&self) -> GetVaultsFilters;
    fn set_vaults(&mut self, filters: GetVaultsFilters);

    /// Update vault filters using a builder function.
    /// Returns an error if the operation fails (e.g., persistence failure).
    fn update_vaults<F>(&mut self, update_fn: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder;
}
