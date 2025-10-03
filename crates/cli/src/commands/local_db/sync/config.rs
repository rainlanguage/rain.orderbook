use anyhow::{anyhow, Context, Result};
use rain_orderbook_app_settings::{
    orderbook::OrderbookCfg,
    yaml::{
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlParsable,
    },
};

pub(crate) async fn load_primary_orderbook_from_commit(
    chain_id: u32,
    commit_hash: &str,
) -> Result<OrderbookCfg> {
    let constants_url = format!(
        "https://raw.githubusercontent.com/rainlanguage/rain.orderbook/{}/packages/webapp/src/lib/constants.ts",
        commit_hash
    );
    let constants_source = fetch_remote_text(&constants_url)
        .await
        .with_context(|| format!("Failed to download constants.ts from {}", constants_url))?;

    let settings_url = extract_settings_url(&constants_source)?;
    let settings_yaml = fetch_remote_text(&settings_url)
        .await
        .with_context(|| format!("Failed to download settings YAML from {}", settings_url))?;

    let orderbook_yaml =
        OrderbookYaml::new(vec![settings_yaml], OrderbookYamlValidation::default())
            .map_err(anyhow::Error::from)?;

    let orderbooks = orderbook_yaml
        .get_orderbooks_by_chain_id(chain_id)
        .map_err(anyhow::Error::from)?;

    // TODO: Support syncing multiple orderbooks for the same network in a single run.
    orderbooks
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No orderbooks configured for chain id {}", chain_id))
}

async fn fetch_remote_text(url: &str) -> Result<String> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Request to {} failed", url))?;
    let response = response
        .error_for_status()
        .with_context(|| format!("Request to {} returned an error status", url))?;
    let body = response
        .text()
        .await
        .with_context(|| format!("Failed to read body from {}", url))?;
    Ok(body)
}

pub(crate) fn extract_settings_url(constants_source: &str) -> Result<String> {
    const KEY: &str = "REMOTE_SETTINGS_URL";

    let key_index = constants_source
        .find(KEY)
        .ok_or_else(|| anyhow!("Unable to locate REMOTE_SETTINGS_URL in constants source"))?;

    let after_key = &constants_source[key_index + KEY.len()..];
    let equals_index = after_key
        .find('=')
        .ok_or_else(|| anyhow!("Unable to locate '=' after REMOTE_SETTINGS_URL"))?;

    let value_segment = after_key[equals_index + 1..].trim_start();
    let mut chars = value_segment.chars();
    let quote = chars
        .next()
        .ok_or_else(|| anyhow!("Unable to parse REMOTE_SETTINGS_URL assignment"))?;

    if quote != '"' && quote != '\'' {
        return Err(anyhow!("Unable to locate REMOTE_SETTINGS_URL quotation"));
    }

    let remainder = &value_segment[1..];
    let closing_index = remainder
        .find(quote)
        .ok_or_else(|| anyhow!("Unable to locate closing quote for REMOTE_SETTINGS_URL"))?;

    Ok(remainder[..closing_index].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_settings_url_success() {
        let source = "export const REMOTE_SETTINGS_URL = 'https://example.com/settings.yaml';";
        let url = extract_settings_url(source).expect("url to be extracted");
        assert_eq!(url, "https://example.com/settings.yaml");
    }

    #[test]
    fn extract_settings_url_missing_key() {
        let source = "export const SOMETHING_ELSE = 'https://example.com';";
        let err = extract_settings_url(source).unwrap_err();
        assert!(err
            .to_string()
            .contains("Unable to locate REMOTE_SETTINGS_URL in constants source"));
    }
}
