use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::{config::Config, config_source::ConfigSource};

#[tauri::command]
pub async fn parse_configstring(text: String) -> CommandResult<ConfigSource> {
    Ok(ConfigSource::try_from_string(text, None).await?.0)
}

#[tauri::command]
pub async fn merge_configstrings(
    dotrain: String,
    config_text: String,
) -> CommandResult<ConfigSource> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str())
        .unwrap_or("")
        .to_string();
    let (mut dotrain_config, config) =
        ConfigSource::try_from_string(frontmatter, Some(config_text)).await?;
    dotrain_config.merge(config)?;
    Ok(dotrain_config)
}

#[tauri::command]
pub fn convert_configstring_to_config(config_string: ConfigSource) -> CommandResult<Config> {
    Ok(config_string.try_into()?)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use alloy::primitives::address;
    use rain_orderbook_app_settings::config_source::{
        ConfigSourceError, NetworkConfigSource, OrderbookConfigSource,
    };
    use url::Url;

    use crate::error::CommandError;

    use super::*;

    #[tokio::test]
    async fn test_parse_configstring_ok() {
        let yaml = r#"
raindex-version: &raindex 123
networks:
    mainnet: &mainnet
        rpc: https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpc: https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
subgraphs: &subgraphs
    mainnet: https://mainnet.subgraph
    testnet: https://testnet.subgraph
orderbooks: &orderbooks
    mainnetOrderbook:
        address: 0xabc0000000000000000000000000000000000001
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
"#;

        let config = parse_configstring(yaml.to_string()).await.unwrap();

        assert_eq!(config.using_networks_from, HashMap::new());

        let expected_networks = HashMap::from([
            (
                "mainnet".to_string(),
                NetworkConfigSource {
                    rpc: Url::parse("https://mainnet.node").unwrap(),
                    chain_id: 1,
                    label: Some("Mainnet".to_string()),
                    network_id: Some(1),
                    currency: Some("ETH".to_string()),
                },
            ),
            (
                "testnet".to_string(),
                NetworkConfigSource {
                    rpc: Url::parse("https://testnet.node").unwrap(),
                    chain_id: 2,
                    label: Some("Testnet".to_string()),
                    network_id: Some(2),
                    currency: Some("ETH".to_string()),
                },
            ),
        ]);

        assert_eq!(config.networks, expected_networks);

        let subgraphs = HashMap::from([
            (
                "mainnet".to_string(),
                Url::parse("https://mainnet.subgraph").unwrap(),
            ),
            (
                "testnet".to_string(),
                Url::parse("https://testnet.subgraph").unwrap(),
            ),
        ]);

        assert_eq!(config.subgraphs, subgraphs);

        let orderbooks = HashMap::from([(
            "mainnetOrderbook".to_string(),
            OrderbookConfigSource {
                address: address!("abc0000000000000000000000000000000000001"),
                network: Some("mainnet".to_string()),
                subgraph: Some("mainnet".to_string()),
                label: Some("Mainnet Orderbook".to_string()),
            },
        )]);

        assert_eq!(config.orderbooks, orderbooks);
        assert_eq!(config.tokens, HashMap::new());
        assert_eq!(config.deployers, HashMap::new());
        assert_eq!(config.orders, HashMap::new());
        assert_eq!(config.scenarios, HashMap::new());
        assert_eq!(config.charts, HashMap::new());
        assert_eq!(config.deployments, HashMap::new());
        assert_eq!(config.metaboards, HashMap::new());
        assert_eq!(config.sentry, None);
        assert_eq!(config.raindex_version, Some("123".to_string()));
        assert_eq!(config.accounts, None);
        assert_eq!(config.gui, None);
    }

    #[tokio::test]
    async fn test_parse_configstring_err_invalid_config() {
        let yaml = r#"
raindex-version: &raindex 123
networks:
    mainnet: &mainnet
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpc: https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
"#;

        let err = parse_configstring(yaml.to_string()).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ConfigSourceError(ConfigSourceError::YamlDeserializerError(msg))
            if msg.to_string().contains("missing field `rpc`")
        ));

        let yaml = r#"
raindex-version: &raindex 123
networks:
    mainnet: &mainnet
        rpc: https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpc: https://testnet.node
        label: Testnet
        network-id: 2
        currency: ETH
"#;

        let err = parse_configstring(yaml.to_string()).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ConfigSourceError(ConfigSourceError::YamlDeserializerError(msg))
            if msg.to_string().contains("missing field `chain-id`")
        ));
    }
}
