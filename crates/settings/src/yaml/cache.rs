use crate::NetworkCfg;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub remote_networks: HashMap<String, NetworkCfg>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            remote_networks: HashMap::new(),
        }
    }

    pub fn update_remote_networks(&mut self, remote_networks: HashMap<String, NetworkCfg>) {
        self.remote_networks = remote_networks;
    }
    pub fn update_remote_network(&mut self, key: String, remote_network: NetworkCfg) {
        self.remote_networks.insert(key, remote_network);
    }
    pub fn get_remote_networks(&self) -> HashMap<String, NetworkCfg> {
        self.remote_networks.clone()
    }
    pub fn get_remote_network(&self, key: &str) -> Option<NetworkCfg> {
        self.remote_networks.get(key).cloned()
    }
}
