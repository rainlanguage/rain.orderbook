use crate::local_db_manifest::{parse_manifest_doc, LocalDbManifest};
use crate::yaml::{load_yaml, YamlError};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum FetchManifestError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    Yaml(#[from] YamlError),
}

pub async fn fetch(url: Url) -> Result<LocalDbManifest, FetchManifestError> {
    let text = reqwest::get(url.to_string()).await?.text().await?;
    let doc = load_yaml(&text)?;
    let manifest = parse_manifest_doc(&doc)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_fetch_manifest_happy_path() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com/dump1"
        end-block: 123
        end-block-hash: "0xabc"
        end-block-time-ms: 1000
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/x-yaml")
                    .body(yaml);
            })
            .await;

        let manifest = fetch(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap();

        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.db_schema_version, 1);
        let net = manifest.networks.get("mainnet").unwrap();
        assert_eq!(net.chain_id, 1);
        assert_eq!(net.orderbooks.len(), 1);
        assert_eq!(
            net.orderbooks[0].address,
            address!("0x0000000000000000000000000000000000000001")
        );
        assert_eq!(net.orderbooks[0].end_block, 123);
        assert_eq!(net.orderbooks[0].end_block_hash, "0xabc");
        assert_eq!(net.orderbooks[0].end_block_time_ms, 1000);

        // find helper
        let found = manifest.find(1, address!("0x0000000000000000000000000000000000000001"));
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_fetch_manifest_unknown_fields_ignored() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 1
extra-root: ignored
networks:
  goerli:
    chain-id: 5
    extra: ignored
    orderbooks:
      - address: "0x0000000000000000000000000000000000000002"
        dump-url: "http://example.com/dump2"
        end-block: 555
        end-block-hash: "0xdef"
        end-block-time-ms: 2000
        extra-ob: ignored
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let manifest = fetch(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap();

        assert!(manifest.networks.contains_key("goerli"));
        let net = manifest.networks.get("goerli").unwrap();
        assert_eq!(net.chain_id, 5);
        assert_eq!(net.orderbooks.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_manifest_invalid_yaml() {
        let server = MockServer::start_async().await;
        let yaml = "manifest-version: [\n"; // malformed

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let err = fetch(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap_err();
        match err {
            // Some malformed YAML inputs are surfaced as ScanError by the loader
            FetchManifestError::Yaml(YamlError::ScanError(_)) => {}
            // In certain cases, incomplete structures may parse into BadValue and
            // be reported later as a field error; accept that as invalid YAML too.
            FetchManifestError::Yaml(YamlError::Field { .. }) => {}
            _ => panic!("expected YAML scan or field error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_manifest_invalid_types_and_values() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 1
networks:
  mainnet:
    chain-id: 1
    orderbooks:
      - address: 123 # invalid type
        dump-url: "not-a-url"
        end-block: 0
        end-block-hash: 999 # invalid type
        end-block-time-ms: 0
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let err = fetch(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap_err();
        match err {
            FetchManifestError::Yaml(YamlError::Field { .. }) => {}
            _ => panic!("expected field error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_manifest_http_error_path() {
        // Use an unsupported scheme to deterministically trigger a reqwest error
        let url = Url::parse("ftp://example.com").unwrap();
        let err = fetch(url).await.unwrap_err();
        match err {
            FetchManifestError::ReqwestError(_) => {}
            other => panic!("expected reqwest error, got {other:?}"),
        }
    }
}
