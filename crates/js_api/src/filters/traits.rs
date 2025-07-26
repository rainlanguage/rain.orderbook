use super::errors::PersistentFilterStoreError;
use crate::rain_orderbook_common::raindex_client::filters::traits::FilterStore;
/// PersistentFilterStore trait for managing filters with persistence
pub trait PersistentFilterStore: FilterStore {
    fn load(&mut self) -> Result<(), PersistentFilterStoreError>;
    fn save(&self) -> Result<(), PersistentFilterStoreError>;
}
