use ethers::middleware::gas_oracle::GasCategory;
use ethers::prelude::gas_oracle::blocknative::Response as BlockNativeResponse;
use reqwest::{header::AUTHORIZATION, Client};
use url::Url;

/// Bloacknative Base Url for fetching blockprices
static BLOCKNATIVE_BLOCKPRICES_URL: &str =
    "https://api.blocknative.com/gasprices/blockprices";

/// Blocknative Gas Oracle.
/// Returns max priority fee and max fee from blocknative api.
///
/// # Arguments
/// * `api_key` - Optional blocknative api key.
/// * `chain_id` - Network Chain Id.
///
pub async fn gas_price_oracle(
    api_key: Option<String>,
    chain_id: u64,
) -> anyhow::Result<(f64, f64)> {
    let client = Client::new();
    let mut url = Url::parse(BLOCKNATIVE_BLOCKPRICES_URL)?;
    url.set_query(Some(format!("chainid={}", chain_id).as_str()));
    let mut request = client.get(url);
    if let Some(api_key) = api_key.as_ref() {
        request = request.header(AUTHORIZATION, api_key);
    }
    let response: BlockNativeResponse = request.send().await?.error_for_status()?.json().await?;
    let fastest = response
        .estimate_from_category(&GasCategory::Fastest)
        .unwrap();
    Ok((fastest.max_priority_fee_per_gas, fastest.max_fee_per_gas))
}
