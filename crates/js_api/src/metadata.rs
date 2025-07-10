use alloy::primitives::hex::FromHexError;
use rain_metaboard_subgraph::metaboard_client::{
    MetaboardSubgraphClient, MetaboardSubgraphClientError,
};
use rain_metadata::{BigInt, RainMetaDocumentV1Item};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Error, Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum MetaDataError {
    #[error("No metadata found for hash {0}")]
    NotFound(String),
    #[error("Subgraph is unavailable: {0}")]
    SubgraphUnavailable(String),
    #[error("Invalid metadata hash format: {0}")]
    InvalidHashFormat(String),
    #[error("Failed to decode metadata: {0}")]
    DecodingError(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Failed to parse metadata content: {0}")]
    ParseError(String),
}
impl_wasm_traits!(MetaDataError);

impl From<MetaboardSubgraphClientError> for MetaDataError {
    fn from(error: MetaboardSubgraphClientError) -> Self {
        match error {
            MetaboardSubgraphClientError::Empty(hash) => MetaDataError::NotFound(hash),
            MetaboardSubgraphClientError::CynicClientError { metahash, source } => {
                MetaDataError::SubgraphUnavailable(format!(
                    "Error for hash {}: {}",
                    metahash, source
                ))
            }
            MetaboardSubgraphClientError::FromHexError { metahash, source } => {
                MetaDataError::DecodingError(format!(
                    "Error decoding hash {}: {}",
                    metahash, source
                ))
            }
        }
    }
}

impl From<FromHexError> for MetaDataError {
    fn from(error: FromHexError) -> Self {
        MetaDataError::InvalidHashFormat(error.to_string())
    }
}

use wasm_bindgen_utils::result::WasmEncodedError;

impl From<MetaDataError> for WasmEncodedError {
    fn from(value: MetaDataError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_string(),
        }
    }
}

/// Client for fetching metadata from the Metaboard subgraph
#[wasm_bindgen]
pub struct RaindexMetaboardClient {
    url: String,
}

#[wasm_export]
impl RaindexMetaboardClient {
    /// Creates a new Metaboard client with the specified subgraph URL
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = RaindexMetaboardClient.new("https://api.thegraph.com/subgraphs/name/your-metaboard-subgraph");
    /// if (result.error) {
    ///   console.error("Failed to create client:", result.error.readableMsg);
    ///   return;
    /// }
    /// const client = result.value;
    /// ```
    #[wasm_export(
        js_name = "new",
        return_description = "A new Metaboard client instance",
        preserve_js_class
    )]
    pub fn new(
        #[wasm_export(param_description = "The URL of the Metaboard subgraph endpoint")]
        url: String,
    ) -> Result<RaindexMetaboardClient, MetaDataError> {
        // Validate URL format
        let _parsed_url = Url::from_str(&url)
            .map_err(|e| MetaDataError::InvalidUrl(format!("Invalid URL '{}': {}", url, e)))?;

        Ok(RaindexMetaboardClient { url })
    }

    /// Fetches all metadata documents by hash from the Metaboard subgraph
    ///
    /// Returns all available metas as RainMetaDocumentV1Item for the given metadata hash.
    /// Multiple documents may exist if the same hash was submitted to different metaboards
    /// or under different subjects.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getAllMetadataByHash("0x1234567890abcdef...");
    /// if (result.error) {
    ///   console.error("Failed to fetch metadata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const docs = result.value;
    /// console.log("Found", docs.length, "meta documents");
    /// ```
    #[wasm_export(
        js_name = "getAllMetadataByHash",
        return_description = "Array of RainMetaDocumentV1Item"
    )]
    pub async fn get_all_metadata_by_hash(
        &self,
        #[wasm_export(
            param_description = "32-byte metadata hash as hex string (with or without 0x prefix)",
            unchecked_param_type = "Hex"
        )]
        metahash: String,
    ) -> Result<Vec<RainMetaDocumentV1Item>, MetaDataError> {
        // Parse the hex hash
        let hash_bytes = self.parse_metahash(&metahash)?;

        // Create client
        let client = self.create_client()?;

        // Fetch metadata from subgraph
        let meta_bytes_vec = client.get_metabytes_by_hash(&hash_bytes).await?;

        // Convert all metadata entries to strings
        let mut contents = Vec::new();
        for meta_bytes in meta_bytes_vec {
            let content = RainMetaDocumentV1Item::cbor_decode(&meta_bytes).map_err(|e| {
                MetaDataError::DecodingError(format!("Failed to decode metadata: {}", e))
            })?;
            contents.push(content);
        }

        Ok(contents)
    }

    /// Fetches all metadata documents by subject from the Metaboard subgraph
    ///
    /// Returns all available metas as RainMetaDocumentV1Item for the given subject.
    /// Multiple documents may exist if many documents was submitted under the same subject.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getAllMetadataBySubject("0x1234567890abcdef...");
    /// if (result.error) {
    ///   console.error("Failed to fetch metadata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const docs = result.value;
    /// console.log("Found", docs.length, "meta documents");
    /// ```
    #[wasm_export(
        js_name = "getAllMetadataBySubject",
        return_description = "Array of RainMetaDocumentV1Item"
    )]
    pub async fn get_all_metadata_by_subject(
        &self,
        #[wasm_export(
            param_description = "Metadata subject as a BigInt string (e.g. '12345678901234567890')",
            unchecked_param_type = "BigInt"
        )]
        subject: String,
    ) -> Result<Vec<RainMetaDocumentV1Item>, MetaDataError> {
        // Parse the hex hash
        let subj = BigInt(subject);

        // Create client
        let client = self.create_client()?;

        // Fetch metadata from subgraph
        let meta_bytes_vec = client.get_metabytes_by_subject(&subj).await?;

        // Convert all metadata entries to strings
        let mut contents = Vec::new();
        for meta_bytes in meta_bytes_vec {
            let content = RainMetaDocumentV1Item::cbor_decode(&meta_bytes).map_err(|e| {
                MetaDataError::DecodingError(format!("Failed to decode metadata: {}", e))
            })?;
            contents.push(content);
        }

        Ok(contents)
    }
}

impl RaindexMetaboardClient {
    /// Helper function to create the subgraph client
    fn create_client(&self) -> Result<MetaboardSubgraphClient, MetaDataError> {
        let parsed_url = Url::from_str(&self.url)
            .map_err(|e| MetaDataError::InvalidUrl(format!("Invalid URL '{}': {}", self.url, e)))?;
        Ok(MetaboardSubgraphClient::new(parsed_url))
    }

    /// Helper function to parse metadata hash from string
    fn parse_metahash(&self, metahash: &str) -> Result<[u8; 32], MetaDataError> {
        // Remove 0x prefix if present
        let hash_str = metahash.strip_prefix("0x").unwrap_or(metahash);

        // Validate length (32 bytes = 64 hex characters)
        if hash_str.len() != 64 {
            return Err(MetaDataError::InvalidHashFormat(format!(
                "Hash must be 32 bytes (64 hex characters), got {} characters",
                hash_str.len()
            )));
        }

        // Parse hex string to bytes
        let bytes = alloy::primitives::hex::decode(hash_str)?;

        // Convert to fixed-size array
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&bytes);

        Ok(hash_array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use httpmock::MockServer;
        use serde_json::json;

        #[test]
        fn test_parse_metahash_with_prefix() {
            let client = RaindexMetaboardClient::new("https://example.com".to_string()).unwrap();
            let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let result = client.parse_metahash(hash).unwrap();

            let expected = [
                0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab,
                0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78,
                0x90, 0xab, 0xcd, 0xef,
            ];
            assert_eq!(result, expected);
        }

        #[test]
        fn test_parse_metahash_without_prefix() {
            let client = RaindexMetaboardClient::new("https://example.com".to_string()).unwrap();
            let hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let result = client.parse_metahash(hash).unwrap();

            let expected = [
                0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab,
                0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78,
                0x90, 0xab, 0xcd, 0xef,
            ];
            assert_eq!(result, expected);
        }

        #[test]
        fn test_parse_metahash_invalid_length() {
            let client = RaindexMetaboardClient::new("https://example.com".to_string()).unwrap();
            let hash = "0x123"; // Too short
            let result = client.parse_metahash(hash);

            assert!(matches!(result, Err(MetaDataError::InvalidHashFormat(_))));
        }

        #[test]
        fn test_parse_metahash_invalid_hex() {
            let client = RaindexMetaboardClient::new("https://example.com".to_string()).unwrap();
            let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg"; // Invalid hex character 'g'
            let result = client.parse_metahash(hash);

            assert!(matches!(result, Err(MetaDataError::InvalidHashFormat(_))));
        }

        #[tokio::test]
        async fn test_get_metadata_by_hash_success() {
            let server = MockServer::start_async().await;
            let client = RaindexMetaboardClient::new(server.url("/")).unwrap();

            // Mock successful response
            server.mock(|when, then| {
                when.body_contains("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
                then.status(200).json_body(json!({
                    "data": {
                        "metaV1S": [
                            {
                                "meta": "0x48656c6c6f20576f726c64", // "Hello World" in hex
                                "metaHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                                "sender": "0x0000000000000000000000000000000000000000",
                                "id": "1",
                                "metaBoard": {
                                    "address": "0x0000000000000000000000000000000000000000"
                                },
                                "subject": "0"
                            }
                        ]
                    }
                }));
            });

            let result = client
                .get_metadata_by_hash(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                        .to_string(),
                )
                .await
                .unwrap();

            assert_eq!(result, "Hello World");
        }

        #[tokio::test]
        async fn test_get_metadata_by_hash_not_found() {
            let server = MockServer::start_async().await;
            let client = RaindexMetaboardClient::new(server.url("/")).unwrap();

            // Mock empty response
            server.mock(|when, then| {
                when.body_contains(
                    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                );
                then.status(200).json_body(json!({
                    "data": {
                        "metaV1S": []
                    }
                }));
            });

            let result = client
                .get_metadata_by_hash(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                        .to_string(),
                )
                .await;

            assert!(matches!(result, Err(MetaDataError::NotFound(_))));
        }

        #[tokio::test]
        async fn test_get_all_metadata_by_hash_multiple_variants() {
            let server = MockServer::start_async().await;
            let client = RaindexMetaboardClient::new(server.url("/")).unwrap();

            // Mock response with multiple variants
            server.mock(|when, then| {
                when.body_contains("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
                then.status(200).json_body(json!({
                    "data": {
                        "metaV1S": [
                            {
                                "meta": "0x48656c6c6f20576f726c64", // "Hello World" in hex
                                "metaHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                                "sender": "0x0000000000000000000000000000000000000000",
                                "id": "1",
                                "metaBoard": {
                                    "address": "0x0000000000000000000000000000000000000000"
                                },
                                "subject": "0"
                            },
                            {
                                "meta": "0x476f6f6462796520576f726c64", // "Goodbye World" in hex
                                "metaHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                                "sender": "0x1111111111111111111111111111111111111111",
                                "id": "2",
                                "metaBoard": {
                                    "address": "0x1111111111111111111111111111111111111111"
                                },
                                "subject": "0"
                            }
                        ]
                    }
                }));
            });

            let result = client
                .get_all_metadata_by_hash(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                        .to_string(),
                )
                .await
                .unwrap();

            assert_eq!(result.len(), 2);
            assert_eq!(result[0], "Hello World");
            assert_eq!(result[1], "Goodbye World");
        }

        #[test]
        fn test_new_client_invalid_url() {
            let result = RaindexMetaboardClient::new("not-a-url".to_string());
            assert!(matches!(result, Err(MetaDataError::InvalidUrl(_))));
        }

        #[tokio::test]
        async fn test_get_all_metadata_by_subject_success() {
            let server = MockServer::start_async().await;
            let client = RaindexMetaboardClient::new(server.url("/")).unwrap();

            // Mock response with multiple documents for the same subject
            server.mock(|when, then| {
                when.body_contains("12345678901234567890"); // The subject BigInt
                then.status(200).json_body(json!({
                    "data": {
                        "metaV1S": [
                            {
                                "meta": "0x48656c6c6f20576f726c64", // "Hello World" in hex
                                "metaHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                                "sender": "0x0000000000000000000000000000000000000000",
                                "id": "1",
                                "metaBoard": {
                                    "address": "0x0000000000000000000000000000000000000000"
                                },
                                "subject": "12345678901234567890"
                            },
                            {
                                "meta": "0x476f6f6462796520576f726c64", // "Goodbye World" in hex
                                "metaHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                                "sender": "0x1111111111111111111111111111111111111111",
                                "id": "2",
                                "metaBoard": {
                                    "address": "0x1111111111111111111111111111111111111111"
                                },
                                "subject": "12345678901234567890"
                            }
                        ]
                    }
                }));
            });

            let result = client
                .get_all_metadata_by_subject("12345678901234567890".to_string())
                .await
                .unwrap();

            assert_eq!(result.len(), 2);
            assert_eq!(result[0], "Hello World");
            assert_eq!(result[1], "Goodbye World");
        }

        #[tokio::test]
        async fn test_get_all_metadata_by_subject_empty() {
            let server = MockServer::start_async().await;
            let client = RaindexMetaboardClient::new(server.url("/")).unwrap();

            // Mock empty response for subject that has no metadata
            server.mock(|when, then| {
                when.body_contains("99999999999999999999"); // Non-existent subject
                then.status(200).json_body(json!({
                    "data": {
                        "metaV1S": []
                    }
                }));
            });

            let result = client
                .get_all_metadata_by_subject("99999999999999999999".to_string())
                .await
                .unwrap();

            assert_eq!(result.len(), 0);
        }
    }
}
