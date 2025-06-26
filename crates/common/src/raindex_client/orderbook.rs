use super::*;

#[wasm_export]
impl RaindexClient {
    /// Get the subgraph identifier for a chain ID
    ///
    /// ## Parameters
    ///
    /// - `chain_id` - The chain ID to get the subgraph identifier for
    ///
    /// ## Returns
    ///
    /// - `String` - The identifier of the subgraph for the given chain ID
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = raindexClient.getSubgraphKeyForChainId(1);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const subgraphKey = result.value;
    /// // Do something with the subgraph key
    /// ```
    #[wasm_export(js_name = "getSubgraphKeyForChainId", unchecked_return_type = "string")]
    pub fn get_subgraph_identifier_for_chain_id(
        &self,
        chain_id: u64,
    ) -> Result<String, RaindexError> {
        self.get_subgraph_key_for_chain_id(chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod non_wasm_tests {
        use super::*;
        use crate::raindex_client::tests::get_test_yaml;

        #[test]
        fn test_get_subgraph_identifier_for_chain_id() {
            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();
            let subgraph_key = raindex_client
                .get_subgraph_identifier_for_chain_id(1)
                .unwrap();
            assert_eq!(subgraph_key, "mainnet");
        }
    }
}
