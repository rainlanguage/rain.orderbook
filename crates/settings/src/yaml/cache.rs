use crate::{NetworkCfg, TokenCfg};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub remote_networks: HashMap<String, NetworkCfg>,
    pub remote_tokens: HashMap<String, TokenCfg>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            remote_networks: HashMap::new(),
            remote_tokens: HashMap::new(),
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

    pub fn update_remote_tokens(&mut self, remote_tokens: HashMap<String, TokenCfg>) {
        self.remote_tokens = remote_tokens;
    }
    pub fn update_remote_token(&mut self, key: String, remote_token: TokenCfg) {
        self.remote_tokens.insert(key, remote_token);
    }
    pub fn get_remote_tokens(&self) -> HashMap<String, TokenCfg> {
        self.remote_tokens.clone()
    }
    pub fn get_remote_token(&self, key: &str) -> Option<TokenCfg> {
        self.remote_tokens.get(key).cloned()
    }
}
