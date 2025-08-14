use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::raindex_client::filters::{
    orders::{builder::OrdersFilterBuilder, filter::GetOrdersFilters},
    vaults::{builder::VaultsFilterBuilder, filter::GetVaultsFilters},
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
    fn update_vaults<F>(&mut self, update_fn: F) -> Result<(), Error>
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder;

    fn get_orders(&self) -> GetOrdersFilters;
    fn set_orders(&mut self, filters: GetOrdersFilters);

    /// Update orders filters using a builder function.
    /// Returns an error if the operation fails (e.g., persistence failure).
    fn update_orders<F>(&mut self, update_fn: F) -> Result<(), Error>
    where
        F: FnOnce(OrdersFilterBuilder) -> OrdersFilterBuilder;
}
