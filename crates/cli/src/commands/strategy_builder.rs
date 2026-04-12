use crate::execute::Execute;
use alloy::primitives::hex;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use std::collections::HashMap;
use url::Url;

#[derive(Parser, Clone)]
pub struct StrategyBuilder {
    #[arg(
        long,
        help = "Registry URL (text file: settings URL on line 1, then 'key url' per order)"
    )]
    registry: String,

    #[arg(long, help = "Order/strategy key from the registry")]
    strategy: String,

    #[arg(long, help = "Deployment key within the strategy")]
    deployment: String,

    #[arg(long, help = "Order owner address")]
    owner: String,

    #[arg(
        long = "set-field",
        value_name = "BINDING=VALUE",
        help = "Set a field binding value (repeatable)"
    )]
    set_fields: Vec<String>,

    #[arg(
        long = "select-token",
        value_name = "KEY=ADDRESS",
        help = "Select a token for a slot (repeatable)"
    )]
    select_tokens: Vec<String>,

    #[arg(
        long = "set-deposit",
        value_name = "TOKEN=AMOUNT",
        help = "Set a deposit amount (repeatable)"
    )]
    set_deposits: Vec<String>,
}

fn parse_key_value_pairs(args: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for arg in args {
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE, got: {arg}"))?;
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}

async fn fetch_text(url: &Url) -> Result<String> {
    let response = reqwest::get(url.as_str()).await?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {status} fetching {url}");
    }
    Ok(response.text().await?)
}

async fn fetch_registry(registry_url: &str) -> Result<(String, HashMap<String, String>)> {
    let registry_url = Url::parse(registry_url)?;
    let content = fetch_text(&registry_url).await?;

    let mut lines = content.lines();
    let settings_url_str = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("registry file is empty"))?
        .trim();
    let settings_url = Url::parse(settings_url_str)
        .map_err(|err| anyhow::anyhow!("invalid settings URL '{settings_url_str}': {err}"))?;

    let mut order_urls = HashMap::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, url_str) = line
            .split_once(' ')
            .ok_or_else(|| anyhow::anyhow!("invalid registry line (expected 'key url'): {line}"))?;
        order_urls.insert(key.to_string(), url_str.trim().to_string());
    }

    let settings = fetch_text(&settings_url).await?;

    Ok((settings, order_urls))
}

impl Execute for StrategyBuilder {
    async fn execute(&self) -> Result<()> {
        let (settings, order_urls) = fetch_registry(&self.registry).await?;

        let order_url_str = order_urls.get(&self.strategy).ok_or_else(|| {
            let available: Vec<_> = order_urls.keys().collect();
            anyhow::anyhow!(
                "strategy '{}' not found in registry. Available: {:?}",
                self.strategy,
                available
            )
        })?;
        let order_url = Url::parse(order_url_str)
            .map_err(|err| anyhow::anyhow!("invalid order URL '{order_url_str}': {err}"))?;
        let dotrain = fetch_text(&order_url).await?;

        let settings_sources = if settings.is_empty() {
            None
        } else {
            Some(vec![settings])
        };

        let mut builder = RaindexOrderBuilder::new_with_deployment(
            dotrain,
            settings_sources,
            self.deployment.clone(),
        )
        .await
        .map_err(|err| {
            anyhow::anyhow!("failed to create order builder: {}", err.to_readable_msg())
        })?;

        let fields = parse_key_value_pairs(&self.set_fields)?;
        for (binding, value) in &fields {
            builder
                .set_field_value(binding.clone(), value.clone())
                .map_err(|err| {
                    anyhow::anyhow!("failed to set field '{binding}': {}", err.to_readable_msg())
                })?;
        }

        let tokens = parse_key_value_pairs(&self.select_tokens)?;
        for (key, address) in &tokens {
            builder
                .set_select_token(key.clone(), address.clone())
                .await
                .map_err(|err| {
                    anyhow::anyhow!("failed to select token '{key}': {}", err.to_readable_msg())
                })?;
        }

        let deposits = parse_key_value_pairs(&self.set_deposits)?;
        for (token, amount) in &deposits {
            builder
                .set_deposit(token.clone(), amount.clone())
                .await
                .map_err(|err| {
                    anyhow::anyhow!("failed to set deposit '{token}': {}", err.to_readable_msg())
                })?;
        }

        let args = builder
            .get_deployment_transaction_args(self.owner.clone())
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to generate deployment calldata: {}",
                    err.to_readable_msg()
                )
            })?;

        for approval in &args.approvals {
            println!("{}:0x{}", approval.token, hex::encode(&approval.calldata));
        }

        println!(
            "{}:0x{}",
            args.orderbook_address,
            hex::encode(&args.deployment_calldata)
        );

        if let Some(meta_call) = &args.emit_meta_call {
            println!("{}:0x{}", meta_call.to, hex::encode(&meta_call.calldata));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        StrategyBuilder::command().debug_assert();
    }

    #[test]
    fn parse_key_value_pairs_valid() {
        let args = vec![
            "max-spread=0.002".to_string(),
            "oracle-key=ETH/USD".to_string(),
        ];
        let map = parse_key_value_pairs(&args).unwrap();
        assert_eq!(map.get("max-spread").unwrap(), "0.002");
        assert_eq!(map.get("oracle-key").unwrap(), "ETH/USD");
    }

    #[test]
    fn parse_key_value_pairs_missing_equals() {
        let args = vec!["no-equals".to_string()];
        let result = parse_key_value_pairs(&args);
        assert!(result.is_err());
    }

    #[test]
    fn parse_key_value_pairs_empty() {
        let args: Vec<String> = vec![];
        let map = parse_key_value_pairs(&args).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn parse_key_value_pairs_value_with_equals() {
        let args = vec!["key=value=with=equals".to_string()];
        let map = parse_key_value_pairs(&args).unwrap();
        assert_eq!(map.get("key").unwrap(), "value=with=equals");
    }
}
