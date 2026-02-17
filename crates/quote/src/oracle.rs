use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV6::{OrderV4, SignedContextV1};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Error types for oracle fetching
#[derive(Debug, thiserror::Error)]
pub enum OracleError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid oracle response: {0}")]
    InvalidResponse(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// JSON response format from an oracle endpoint.
/// Maps directly to `SignedContextV1` in the orderbook contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// The signer address (EIP-191 signer of the context data)
    pub signer: Address,
    /// The signed context data as bytes32[] values
    pub context: Vec<FixedBytes<32>>,
    /// The EIP-191 signature over keccak256(abi.encodePacked(context))
    pub signature: Bytes,
}

impl From<OracleResponse> for SignedContextV1 {
    fn from(resp: OracleResponse) -> Self {
        SignedContextV1 {
            signer: resp.signer,
            context: resp.context,
            signature: resp.signature,
        }
    }
}

/// Encode the POST body for an oracle request.
///
/// The body is `abi.encode(OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)`.
pub fn encode_oracle_body(
    order: &OrderV4,
    input_io_index: u32,
    output_io_index: u32,
    counterparty: Address,
) -> Vec<u8> {
    (
        order.clone(),
        U256::from(input_io_index),
        U256::from(output_io_index),
        counterparty,
    )
        .abi_encode()
}

/// Fetch signed context from an oracle endpoint via POST.
///
/// The endpoint receives an ABI-encoded body containing the order details
/// that will be used for calculateOrderIO:
/// `abi.encode(OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)`
///
/// The endpoint must respond with a JSON body matching `OracleResponse`.
pub async fn fetch_signed_context(
    url: &str,
    body: Vec<u8>,
) -> Result<SignedContextV1, OracleError> {
    let builder = Client::builder();
    #[cfg(not(target_family = "wasm"))]
    let builder = builder.timeout(std::time::Duration::from_secs(10));
    let client = builder.build()?;

    let response: OracleResponse = client
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response.into())
}

/// Extract the oracle URL from an SgOrder's meta, if present.
///
/// Parses the meta bytes and looks for a `RaindexSignedContextOracleV1` entry.
/// Returns `None` if meta is absent, unparseable, or doesn't contain an oracle entry.
pub fn extract_oracle_url(order: &SgOrder) -> Option<String> {
    use rain_metadata::types::raindex_signed_context_oracle::RaindexSignedContextOracleV1;
    use rain_metadata::RainMetaDocumentV1Item;

    let meta = order.meta.as_ref()?;
    let decoded = alloy::hex::decode(&meta.0).ok()?;
    let items = RainMetaDocumentV1Item::cbor_decode(&decoded).ok()?;
    let oracle = RaindexSignedContextOracleV1::find_in_items(&items).ok()??;
    Some(oracle.url().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, FixedBytes};

    #[test]
    fn test_oracle_response_to_signed_context() {
        let ctx_val = FixedBytes::<32>::from([0x2a; 32]);
        let response = OracleResponse {
            signer: address!("0x1234567890123456789012345678901234567890"),
            context: vec![ctx_val],
            signature: Bytes::from(vec![0xaa, 0xbb, 0xcc]),
        };

        let signed: SignedContextV1 = response.into();
        assert_eq!(
            signed.signer,
            address!("0x1234567890123456789012345678901234567890")
        );
        assert_eq!(signed.context.len(), 1);
        assert_eq!(signed.context[0], ctx_val);
        assert_eq!(signed.signature, Bytes::from(vec![0xaa, 0xbb, 0xcc]));
    }

    #[tokio::test]
    async fn test_fetch_signed_context_invalid_url() {
        let result = fetch_signed_context("not-a-url", vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_signed_context_unreachable() {
        let result = fetch_signed_context("http://127.0.0.1:1/oracle", vec![]).await;
        assert!(result.is_err());
    }
}
