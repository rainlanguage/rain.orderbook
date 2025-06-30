use alloy::network::AnyNetwork;
use alloy::providers::{
    fillers::FillProvider, utils::JoinedRecommendedFillers, ProviderBuilder, RootProvider,
};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::Http;
#[cfg(not(target_family = "wasm"))]
use alloy::transports::layers::FallbackLayer;
#[cfg(not(target_family = "wasm"))]
use std::num::NonZeroUsize;
use thiserror::Error;
#[cfg(not(target_family = "wasm"))]
use tower::ServiceBuilder;
use url::Url;

pub type ReadProvider =
    FillProvider<JoinedRecommendedFillers, RootProvider<AnyNetwork>, AnyNetwork>;

#[derive(Error, Debug)]
pub enum ReadProviderError {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("No RPC URLs provided")]
    NoRpcs,
}

#[cfg(target_family = "wasm")]
pub fn mk_read_provider(rpcs: &[&str]) -> Result<ReadProvider, ReadProviderError> {
    let rpc = rpcs.first().ok_or(ReadProviderError::NoRpcs)?;
    let transport = Http::new(Url::parse(rpc)?);
    let client = RpcClient::builder().transport(transport, false);
    let provider = ProviderBuilder::new_with_network::<AnyNetwork>().connect_client(client);
    Ok(provider)
}

#[cfg(not(target_family = "wasm"))]
pub fn mk_read_provider(rpcs: &[&str]) -> Result<ReadProvider, ReadProviderError> {
    let size = rpcs.len();

    let fallback_layer = FallbackLayer::default()
        .with_active_transport_count(NonZeroUsize::new(size).ok_or(ReadProviderError::NoRpcs)?);

    let transports = rpcs
        .iter()
        .map(|rpc| Ok::<_, ReadProviderError>(Http::new(Url::parse(rpc)?)))
        .collect::<Result<Vec<_>, _>>()?;

    let transport = ServiceBuilder::new()
        .layer(fallback_layer)
        .service(transports);
    let client = RpcClient::builder().transport(transport, false);
    let provider = ProviderBuilder::new_with_network::<AnyNetwork>().connect_client(client);

    Ok(provider)
}
