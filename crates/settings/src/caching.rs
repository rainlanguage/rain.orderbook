use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

use crate::NetworkCfg;

#[derive(Error, Debug, PartialEq)]
pub enum CacheError {
    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Network not found: {0}")]
    NetworkNotFound(String),
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub trait CacheStrategy {
    fn get_networks(&self) -> Result<HashMap<String, NetworkCfg>, CacheError>;
    fn set_networks(&self, networks: HashMap<String, NetworkCfg>) -> Result<(), CacheError>;
    fn get_network(&self, key: &str) -> Result<NetworkCfg, CacheError>;
    fn set_network(&self, key: &str, network: NetworkCfg) -> Result<(), CacheError>;
    fn clear(&self) -> Result<(), CacheError>;
}

#[cfg(target_family = "wasm")]
pub struct WasmCache {
    storage: web_sys::Storage,
}

#[cfg(target_family = "wasm")]
impl CacheStrategy for WasmCache {
    fn get_networks(&self) -> Result<HashMap<String, NetworkCfg>, CacheError> {
        match self.storage.get_item("networks") {
            Ok(Some(value)) => serde_json::from_str(&value)
                .map_err(|e| CacheError::SerializationError(e.to_string())),
            Ok(None) => Ok(HashMap::new()),
            Err(e) => Err(CacheError::StorageError(e.as_string().unwrap_or_default())),
        }
    }

    fn set_networks(&self, networks: HashMap<String, NetworkCfg>) -> Result<(), CacheError> {
        let serialized = serde_json::to_string(&networks)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        self.storage
            .set_item("networks", &serialized)
            .map_err(|e| CacheError::StorageError(e.as_string().unwrap_or_default()))?;

        Ok(())
    }

    fn get_network(&self, key: &str) -> Result<NetworkCfg, CacheError> {
        let networks = self.get_networks()?;
        networks
            .get(key)
            .cloned()
            .ok_or_else(|| CacheError::NetworkNotFound(key.to_string()))
    }

    fn set_network(&self, key: &str, network: NetworkCfg) -> Result<(), CacheError> {
        let mut networks = self.get_networks()?;
        networks.insert(key.to_string(), network);
        self.set_networks(networks)
    }

    fn clear(&self) -> Result<(), CacheError> {
        self.storage
            .remove_item("networks")
            .map_err(|e| CacheError::StorageError(e.as_string().unwrap_or_default()))?;
        Ok(())
    }
}

#[cfg(not(target_family = "wasm"))]
pub struct NativeCache {
    networks: RwLock<HashMap<String, NetworkCfg>>,
}

#[cfg(not(target_family = "wasm"))]
impl NativeCache {
    pub fn new() -> Self {
        Self {
            networks: RwLock::new(HashMap::new()),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl CacheStrategy for NativeCache {
    fn get_networks(&self) -> Result<HashMap<String, NetworkCfg>, CacheError> {
        self.networks
            .read()
            .map_err(|_| CacheError::ReadLockError)
            .map(|networks| networks.clone())
    }

    fn set_networks(&self, networks: HashMap<String, NetworkCfg>) -> Result<(), CacheError> {
        let mut cache = self
            .networks
            .write()
            .map_err(|_| CacheError::WriteLockError)?;
        *cache = networks;
        Ok(())
    }

    fn get_network(&self, key: &str) -> Result<NetworkCfg, CacheError> {
        let networks = self
            .networks
            .read()
            .map_err(|_| CacheError::ReadLockError)?;
        networks
            .get(key)
            .cloned()
            .ok_or_else(|| CacheError::NetworkNotFound(key.to_string()))
    }

    fn set_network(&self, key: &str, network: NetworkCfg) -> Result<(), CacheError> {
        let mut networks = self.get_networks()?;
        networks.insert(key.to_string(), network);
        self.set_networks(networks)
    }

    fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self
            .networks
            .write()
            .map_err(|_| CacheError::WriteLockError)?;
        cache.clear();
        Ok(())
    }
}

// Factory function remains the same
pub fn get_cache() -> Box<dyn CacheStrategy> {
    #[cfg(target_family = "wasm")]
    {
        Box::new(WasmCache::new())
    }
    #[cfg(not(target_family = "wasm"))]
    {
        Box::new(NativeCache::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cache() {
        let cache = NativeCache::new();
        assert_eq!(cache.get_networks().unwrap(), HashMap::new());
    }

    #[test]
    fn test_set_and_get_networks() {
        let cache = NativeCache::new();
        cache
            .set_network("test_network", NetworkCfg::dummy())
            .unwrap();

        let result = cache.get_network("test_network").unwrap();
        assert_eq!(result, NetworkCfg::dummy());

        let results = cache.get_networks().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results.get("test_network").unwrap(), &NetworkCfg::dummy());
    }

    #[test]
    fn test_get_nonexistent_network() {
        let cache = NativeCache::new();
        let result = cache.get_network("nonexistent_network");
        assert_eq!(
            result,
            Err(CacheError::NetworkNotFound(
                "nonexistent_network".to_string()
            ))
        );
    }

    #[test]
    fn test_clear_cache() {
        let cache = NativeCache::new();
        cache
            .set_network("test_network", NetworkCfg::dummy())
            .unwrap();
        assert_eq!(cache.get_networks().unwrap().len(), 1);

        cache.clear().unwrap();
        assert_eq!(cache.get_networks().unwrap(), HashMap::new());
    }
}
