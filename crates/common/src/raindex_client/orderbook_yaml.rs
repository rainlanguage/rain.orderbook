use super::*;
use rain_orderbook_app_settings::{
    accounts::AccountCfg, network::NetworkCfg, orderbook::OrderbookCfg,
};
use std::collections::HashMap;

#[wasm_export]
impl RaindexClient {
    /// Retrieves a list of unique chain IDs from all configured networks
    ///
    /// Extracts and returns all unique blockchain network chain IDs that are available
    /// in the current orderbook configuration.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = client.getUniqueChainIds();
    /// if (result.error) {
    ///   console.error("Error getting chain IDs:", result.error.readableMsg);
    ///   return;
    /// }
    /// const chainIds = result.value;
    /// console.log("Available chains:", chainIds);
    /// ```
    #[wasm_export(
        js_name = "getUniqueChainIds",
        return_description = "Returns a list of unique chain IDs from all available networks.",
        unchecked_return_type = "number[]"
    )]
    pub fn get_unique_chain_ids(&self) -> Result<Vec<u32>, RaindexError> {
        let networks = self.get_all_networks()?;
        Ok(networks.values().map(|cfg| cfg.chain_id).collect())
    }

    /// Retrieves all available networks with their configurations
    ///
    /// Returns a comprehensive map of all network configurations available in the orderbook YAML.
    /// Each entry contains detailed network information including RPC endpoints and chain-specific settings.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = client.getAllNetworks();
    /// if (result.error) {
    ///   console.error("Error getting networks:", result.error.readableMsg);
    ///   return;
    /// }
    /// const networks = result.value;
    /// for (const [key, config] of networks) {
    ///   console.log(`Network key: ${key}, Chain ID: ${config.chainId}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getAllNetworks",
        return_description = "Returns a map of all available networks with their configurations. Keys are network names, values are NetworkCfg objects.",
        unchecked_return_type = "Map<string, NetworkCfg>"
    )]
    pub fn get_all_networks(&self) -> Result<HashMap<String, NetworkCfg>, RaindexError> {
        Ok(self.orderbook_yaml.get_networks()?)
    }

    /// Retrieves network configuration for a specific chain ID
    ///
    /// Finds and returns the network configuration that matches the provided chain ID.
    /// This is useful when you need to access network-specific settings like RPC URLs.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = client.getNetworkByChainId(1); // Ethereum mainnet
    /// if (result.error) {
    ///   console.error("Network not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const networkConfig = result.value;
    /// console.log(`Found network: ${networkConfig}`);
    /// ```
    #[wasm_export(
        js_name = "getNetworkByChainId",
        return_description = "Returns the configuration for a specific network identified by its chain ID",
        unchecked_return_type = "NetworkCfg"
    )]
    pub fn get_network_by_chain_id(
        &self,
        #[wasm_export(
            js_name = "chainId",
            param_description = "The blockchain network ID to retrieve the configuration for"
        )]
        chain_id: u32,
    ) -> Result<NetworkCfg, RaindexError> {
        Ok(self.orderbook_yaml.get_network_by_chain_id(chain_id)?)
    }

    /// Retrieves orderbook configuration by contract address
    ///
    /// Finds and returns the orderbook configuration that matches the provided contract address.
    /// This allows you to access orderbook-specific settings including subgraph endpoints,
    /// network information, and other details.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = client.getOrderbookByAddress("0x1234567890123456789012345678901234567890");
    /// if (result.error) {
    ///   console.error("Orderbook not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orderbookConfig = result.value;
    /// console.log(`Found orderbook ${orderbookConfig}`);
    /// ```
    #[wasm_export(
        js_name = "getOrderbookByAddress",
        return_description = "Returns the configuration for a specific orderbook identified by its address",
        unchecked_return_type = "OrderbookCfg"
    )]
    pub fn get_orderbook_by_address_wasm_binding(
        &self,
        #[wasm_export(
            param_description = "The address of the orderbook to retrieve the configuration for"
        )]
        address: String,
    ) -> Result<OrderbookCfg, RaindexError> {
        let address = Address::from_str(&address)?;
        Ok(self.orderbook_yaml.get_orderbook_by_address(address)?)
    }

    /// Checks if Sentry error tracking is enabled in the YAML configuration
    ///
    /// Returns `true` if Sentry is enabled, otherwise returns `false`.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const isEnabled = client.isSentryEnabled();
    /// if (isEnabled.error) {
    ///   console.error("Error checking Sentry status:", isEnabled.error.readableMsg);
    ///   return;
    /// }
    /// console.log("Is Sentry enabled?", isEnabled.value);
    /// ```
    #[wasm_export(
        js_name = "isSentryEnabled",
        return_description = "Returns whether Sentry is enabled in the YAML configuration.",
        unchecked_return_type = "boolean"
    )]
    pub fn is_sentry_enabled(&self) -> Result<bool, RaindexError> {
        let sentry = self.orderbook_yaml.get_sentry()?;
        Ok(sentry.unwrap_or(false))
    }

    /// Retrieves all accounts from the orderbook YAML configuration
    ///
    /// Returns a map of account configurations where the keys are account names
    /// and the values are `AccountCfg` objects containing account details.
    ///
    /// ## Examples
    /// ///
    /// ```javascript
    /// const result = client.getAllAccounts();
    /// if (result.error) {
    ///   console.error("Error getting accounts:", result.error.readableMsg);
    ///   return;
    /// }
    /// const accounts = result.value;
    /// for (const [name, account] of accounts) {
    ///   console.log(`Account name: ${name}, Address: ${account.address}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getAllAccounts",
        return_description = "Returns the list of accounts from the orderbook YAML configuration.",
        unchecked_return_type = "Map<string, AccountCfg>"
    )]
    pub fn get_all_accounts(&self) -> Result<HashMap<String, AccountCfg>, RaindexError> {
        Ok(self.orderbook_yaml.get_accounts()?)
    }
}
impl RaindexClient {
    pub fn get_orderbook_by_address(&self, address: Address) -> Result<OrderbookCfg, RaindexError> {
        Ok(self.orderbook_yaml.get_orderbook_by_address(address)?)
    }
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::tests::get_test_yaml;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_get_unique_chain_ids() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();
        let result = client.get_unique_chain_ids().unwrap();

        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert!(result.contains(&1));
        assert!(result.contains(&137));
    }

    #[wasm_bindgen_test]
    fn test_get_all_networks() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();
        let result = client.get_all_networks().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("mainnet"));
        assert!(result.contains_key("polygon"));

        let mainnet = result.get("mainnet").unwrap();
        assert_eq!(mainnet.chain_id, 1);
        assert_eq!(mainnet.label, Some("Ethereum Mainnet".to_string()));

        let polygon = result.get("polygon").unwrap();
        assert_eq!(polygon.chain_id, 137);
        assert_eq!(polygon.label, Some("Polygon Mainnet".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_get_network_by_chain_id() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();

        let mainnet = client.get_network_by_chain_id(1).unwrap();
        assert_eq!(mainnet.chain_id, 1);
        assert_eq!(mainnet.label, Some("Ethereum Mainnet".to_string()));

        let polygon = client.get_network_by_chain_id(137).unwrap();
        assert_eq!(polygon.chain_id, 137);
        assert_eq!(polygon.label, Some("Polygon Mainnet".to_string()));

        let result = client.get_network_by_chain_id(999);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_get_orderbook_by_address_wasm_binding() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();

        let mainnet_address = "0x1234567890123456789012345678901234567890".to_string();
        let mainnet_orderbook = client
            .get_orderbook_by_address_wasm_binding(mainnet_address.clone())
            .unwrap();
        assert_eq!(
            mainnet_orderbook.address.to_string().to_lowercase(),
            mainnet_address.to_lowercase()
        );

        let polygon_address = "0x0987654321098765432109876543210987654321".to_string();
        let polygon_orderbook = client
            .get_orderbook_by_address_wasm_binding(polygon_address.clone())
            .unwrap();
        assert_eq!(
            polygon_orderbook.address.to_string().to_lowercase(),
            polygon_address.to_lowercase()
        );

        let result = client.get_orderbook_by_address_wasm_binding(
            "0x1111111111111111111111111111111111111111".to_string(),
        );
        assert!(result.is_err());

        let result = client.get_orderbook_by_address_wasm_binding("invalid_address".to_string());
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_is_sentry_enabled() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();
        let result = client.is_sentry_enabled().unwrap();

        assert!(!result);
    }

    #[wasm_bindgen_test]
    fn test_get_all_accounts() {
        let yaml = get_test_yaml(
            "http://localhost:3001",
            "http://localhost:3002",
            "http://localhost:3003",
            "http://localhost:3004",
        );
        let client = RaindexClient::new(vec![yaml], None).unwrap();
        let result = client.get_all_accounts().unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains_key("alice"));
        assert!(result.contains_key("bob"));
        assert!(result.contains_key("charlie"));

        let alice = result.get("alice").unwrap();
        assert_eq!(
            alice.address,
            Address::from_str("0x742d35Cc6634C0532925a3b8D4Fd2d3dB2d4D7fA").unwrap()
        );

        let bob = result.get("bob").unwrap();
        assert_eq!(
            bob.address,
            Address::from_str("0x8ba1f109551bD432803012645aac136c0c8D2e80").unwrap()
        );

        let charlie = result.get("charlie").unwrap();
        assert_eq!(
            charlie.address,
            Address::from_str("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5").unwrap()
        );
    }
}
