use super::errors::PersistentFilterStoreError;
use crate::rain_orderbook_common::raindex_client::filters::traits::FilterStore;
/// PersistentFilterStore trait for managing filters with persistence
pub trait PersistentFilterStore: FilterStore {
    /// Loads persisted filters into this store.
    /// Implementations should follow the agreed priority (e.g., URL params > localStorage),
    /// and be safe to call multiple times.
    fn load(&mut self) -> Result<(), PersistentFilterStoreError>;
    /// Persists the current filters.
    /// If it makes sense, implementations may save state to multiple persistent storages
    /// e.g. to URLParams and LocalStorage
    fn save(&self) -> Result<(), PersistentFilterStoreError>;
}
