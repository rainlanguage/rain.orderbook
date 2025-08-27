use alloy::network::AnyNetwork;
use alloy::providers::{
    fillers::FillProvider, utils::JoinedRecommendedFillers, ProviderBuilder, RootProvider,
};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::Http;
use alloy::transports::layers::FallbackLayer;
use std::num::NonZeroUsize;
use thiserror::Error;
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

pub fn mk_read_provider(rpcs: &[Url]) -> Result<ReadProvider, ReadProviderError> {
    let size = rpcs.len();

    let fallback_layer = FallbackLayer::default()
        .with_active_transport_count(NonZeroUsize::new(size).ok_or(ReadProviderError::NoRpcs)?);

    let transports = rpcs
        .iter()
        .map(|rpc| Ok::<_, ReadProviderError>(Http::new(rpc.clone())))
        .collect::<Result<Vec<_>, _>>()?;

    let transport = ServiceBuilder::new()
        .layer(fallback_layer)
        .service(transports);
    let client = RpcClient::builder().transport(transport, false);
    let provider = ProviderBuilder::new_with_network::<AnyNetwork>().connect_client(client);

    Ok(provider)
}
