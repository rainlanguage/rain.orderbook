use alloy::primitives::{Address, Bytes, FixedBytes};
use rain_orderbook_bindings::IOrderBookV6::SignedContextV1;
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

/// Fetch signed context from an oracle endpoint via POST.
///
/// The endpoint receives an ABI-encoded body containing the order details
/// that will be used for calculateOrderIO:
/// `abi.encode(OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)`
///
/// The endpoint must respond with a JSON body matching `OracleResponse`.
///
/// If `body` is None, falls back to a GET request (for simple oracles that
/// don't need order details).
pub async fn fetch_signed_context(
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<SignedContextV1, OracleError> {
    let builder = Client::builder();
    #[cfg(not(target_family = "wasm"))]
    let builder = builder.timeout(std::time::Duration::from_secs(10));
    let client = builder.build()?;

    let request = match body {
        Some(data) => client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .body(data),
        None => client.get(url),
    };

    let response: OracleResponse = request
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response.into())
}

/// Fetch signed contexts for multiple oracle URLs concurrently.
///
/// Returns a vec of results - one per URL. Failed fetches return errors
/// rather than failing the entire batch, so callers can decide how to handle
/// partial failures.
pub async fn fetch_signed_contexts(
    urls: &[String],
    body: Option<Vec<u8>>,
) -> Vec<Result<SignedContextV1, OracleError>> {
    let futures: Vec<_> = urls
        .iter()
        .map(|url| fetch_signed_context(url, body.clone()))
        .collect();

    futures::future::join_all(futures).await
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
        let result = fetch_signed_context("not-a-url", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_signed_context_unreachable() {
        let result = fetch_signed_context("http://127.0.0.1:1/oracle", None).await;
        assert!(result.is_err());
    }
}
