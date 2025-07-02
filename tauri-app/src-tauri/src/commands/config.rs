use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::{
    config::Config, config_source::ConfigSource, new_config::NewConfig,
};

#[tauri::command]
pub async fn parse_configstring(text: String) -> CommandResult<ConfigSource> {
    Ok(ConfigSource::try_from_string(text, None).await?.0)
}

#[tauri::command]
pub async fn parse_new_configstring(text: String, validate: bool) -> CommandResult<NewConfig> {
    Ok(NewConfig::try_from_yaml(vec![text], validate)?)
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
    use alloy::primitives::address;
    use std::collections::HashMap;
    use strict_yaml_rust::StrictYaml;
    use url::Url;

    use super::*;
    use crate::error::CommandError;
    use rain_orderbook_app_settings::{
        config_source::{ConfigSourceError, NetworkConfigSource, OrderbookConfigSource},
        merge::MergeError,
        orderbook::ParseOrderbookConfigSourceError,
        spec_version::SpecVersion,
        ParseConfigSourceError,
    };

    #[tokio::test]
    async fn test_parse_configstring_and_convert_configstring_to_config() {
        let yaml = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpcs:
            - https://testnet.node
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
"#,
            spec_version = SpecVersion::current()
        );

        let config_src = parse_configstring(yaml.to_string()).await.unwrap();

        assert_eq!(config_src.using_networks_from, HashMap::new());

        let expected_network_config_sources = HashMap::from([
            (
                "mainnet".to_string(),
                NetworkConfigSource {
                    rpcs: vec![Url::parse("https://mainnet.node").unwrap()],
                    chain_id: 1,
                    label: Some("Mainnet".to_string()),
                    network_id: Some(1),
                    currency: Some("ETH".to_string()),
                },
            ),
            (
                "testnet".to_string(),
                NetworkConfigSource {
                    rpcs: vec![Url::parse("https://testnet.node").unwrap()],
                    chain_id: 2,
                    label: Some("Testnet".to_string()),
                    network_id: Some(2),
                    currency: Some("ETH".to_string()),
                },
            ),
        ]);
        assert_eq!(config_src.networks, expected_network_config_sources);

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

        assert_eq!(config_src.subgraphs, subgraphs);

        let orderbooks = HashMap::from([(
            "mainnetOrderbook".to_string(),
            OrderbookConfigSource {
                address: address!("abc0000000000000000000000000000000000001"),
                network: Some("mainnet".to_string()),
                subgraph: Some("mainnet".to_string()),
                label: Some("Mainnet Orderbook".to_string()),
            },
        )]);

        assert_eq!(config_src.orderbooks, orderbooks);
        assert_eq!(config_src.tokens, HashMap::new());
        assert_eq!(config_src.deployers, HashMap::new());
        assert_eq!(config_src.orders, HashMap::new());
        assert_eq!(config_src.scenarios, HashMap::new());
        assert_eq!(config_src.charts, HashMap::new());
        assert_eq!(config_src.deployments, HashMap::new());
        assert_eq!(config_src.metaboards, HashMap::new());
        assert_eq!(config_src.sentry, None);
        assert_eq!(config_src.version, "1".to_string());
        assert_eq!(config_src.accounts, None);
        assert_eq!(config_src.gui, None);

        let err = convert_configstring_to_config(ConfigSource {
            networks: HashMap::new(),
            ..config_src.clone()
        })
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::ParseConfigSourceError(
                ParseConfigSourceError::ParseOrderbookConfigSourceError(
                    ParseOrderbookConfigSourceError::NetworkNotFoundError(msg)
                )
            ) if &msg == "mainnet"
        ));

        let config = convert_configstring_to_config(config_src).unwrap();

        // Inspect config networks
        assert_eq!(config.networks.len(), expected_network_config_sources.len());

        let mainnet_network_cfg = config.networks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_network_cfg.rpcs,
            vec![Url::parse("https://mainnet.node").unwrap()]
        );
        assert_eq!(mainnet_network_cfg.chain_id, 1);
        assert_eq!(mainnet_network_cfg.label, Some("Mainnet".to_string()));
        assert_eq!(mainnet_network_cfg.network_id, Some(1));
        assert_eq!(mainnet_network_cfg.currency, Some("ETH".to_string()));
        assert_eq!(mainnet_network_cfg.key, "mainnet");

        let mainnet_doc = mainnet_network_cfg.document.read().unwrap().to_owned();
        assert_eq!(mainnet_doc, StrictYaml::String("".to_string()));

        let testnet_network_cfg = config.networks.get("testnet").unwrap();
        assert_eq!(
            testnet_network_cfg.rpcs,
            vec![Url::parse("https://testnet.node").unwrap()]
        );
        assert_eq!(testnet_network_cfg.chain_id, 2);
        assert_eq!(testnet_network_cfg.label, Some("Testnet".to_string()));
        assert_eq!(testnet_network_cfg.network_id, Some(2));
        assert_eq!(testnet_network_cfg.currency, Some("ETH".to_string()));
        assert_eq!(testnet_network_cfg.key, "testnet");

        let testnet_doc = testnet_network_cfg.document.read().unwrap().to_owned();
        assert_eq!(testnet_doc, StrictYaml::String("".to_string()));

        // Inspect config subgraphs
        assert_eq!(config.subgraphs.len(), subgraphs.len());

        let mainnet_subgraph_cfg = config.subgraphs.get("mainnet").unwrap();
        assert_eq!(
            mainnet_subgraph_cfg.url,
            Url::parse("https://mainnet.subgraph").unwrap()
        );
        assert_eq!(mainnet_subgraph_cfg.key, "mainnet");

        let mainnet_subgraph_doc = mainnet_subgraph_cfg.document.read().unwrap().to_owned();
        assert_eq!(mainnet_subgraph_doc, StrictYaml::String("".to_string()));

        let testnet_subgraph_cfg = config.subgraphs.get("testnet").unwrap();
        assert_eq!(
            testnet_subgraph_cfg.url,
            Url::parse("https://testnet.subgraph").unwrap()
        );
        assert_eq!(testnet_subgraph_cfg.key, "testnet");

        let testnet_subgraph_doc = testnet_subgraph_cfg.document.read().unwrap().to_owned();
        assert_eq!(testnet_subgraph_doc, StrictYaml::String("".to_string()));

        // Inspect metaboards
        assert_eq!(config.metaboards, HashMap::new());

        // Inspect orderbooks
        assert_eq!(config.orderbooks.len(), orderbooks.len());

        let mainnet_orderbook_cfg = config.orderbooks.get("mainnetOrderbook").unwrap();
        assert_eq!(
            mainnet_orderbook_cfg.address,
            address!("abc0000000000000000000000000000000000001")
        );

        assert_eq!(mainnet_orderbook_cfg.network.chain_id, 1);
        assert_eq!(
            mainnet_orderbook_cfg.network.label,
            Some("Mainnet".to_string())
        );
        assert_eq!(mainnet_orderbook_cfg.network.network_id, Some(1));
        assert_eq!(
            mainnet_orderbook_cfg.network.currency,
            Some("ETH".to_string())
        );
        assert_eq!(mainnet_orderbook_cfg.network.key, "mainnet");

        let mainnet_orderbook_network_doc = mainnet_orderbook_cfg
            .network
            .document
            .read()
            .unwrap()
            .to_owned();
        assert_eq!(
            mainnet_orderbook_network_doc,
            StrictYaml::String("".to_string())
        );

        assert_eq!(
            mainnet_orderbook_cfg.subgraph.url,
            Url::parse("https://mainnet.subgraph").unwrap()
        );
        assert_eq!(mainnet_orderbook_cfg.subgraph.key, "mainnet");

        let mainnet_orderbook_subgraph_doc = mainnet_orderbook_cfg
            .subgraph
            .document
            .read()
            .unwrap()
            .to_owned();
        assert_eq!(
            mainnet_orderbook_subgraph_doc,
            StrictYaml::String("".to_string())
        );

        assert_eq!(
            mainnet_orderbook_cfg.label,
            Some("Mainnet Orderbook".to_string())
        );

        let mainnet_orderbook_doc = mainnet_orderbook_cfg.document.read().unwrap().to_owned();
        assert_eq!(mainnet_orderbook_doc, StrictYaml::String("".to_string()));

        // Inspect tokens
        assert_eq!(config.tokens, HashMap::new());
        // Inspect deployers
        assert_eq!(config.deployers, HashMap::new());
        // Inspect orders
        assert_eq!(config.orders, HashMap::new());
        // Inspect scenarios
        assert_eq!(config.scenarios, HashMap::new());
        // Inspect charts
        assert_eq!(config.charts, HashMap::new());
        // Inspect deployments
        assert_eq!(config.deployments, HashMap::new());
        // Inspect sentry
        assert_eq!(config.sentry, None);
        // Inspect spec_version
        assert_eq!(config.version, "1".to_string());
        // Inspect accounts
        assert_eq!(config.accounts, None);
        // Inspect gui
    }

    #[tokio::test]
    async fn test_parse_configstring_err_invalid_config() {
        let yaml = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpcs:
            - https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
"#,
            spec_version = SpecVersion::current()
        );

        let err = parse_configstring(yaml.to_string()).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ConfigSourceError(ConfigSourceError::YamlDeserializerError(msg))
            if msg.to_string().contains("missing field `rpcs`")
        ));

        let yaml = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpcs:
            - https://testnet.node
        label: Testnet
        network-id: 2
        currency: ETH
"#,
            spec_version = SpecVersion::current()
        );

        let err = parse_configstring(yaml.to_string()).await.unwrap_err();
        assert!(matches!(
            err,
            CommandError::ConfigSourceError(ConfigSourceError::YamlDeserializerError(msg))
            if msg.to_string().contains("missing field `chain-id`")
        ));
    }

    #[tokio::test]
    async fn test_merge_configstrings_ok() {
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
subgraphs:
    mainnet: https://mainnet.subgraph
orderbooks:
    mainnetOrderbook:
        address: 0xabc0000000000000000000000000000000000001
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
---
"#,
            spec_version = SpecVersion::current()
        );

        let config_text = format!(
            r#"
version: {spec_version}
networks:
    testnet: &testnet
        rpcs:
            - https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
subgraphs:
    testnet: https://testnet.subgraph
orderbooks:
    testnetOrderbook:
        address: 0xdef0000000000000000000000000000000000002
        network: testnet
        subgraph: testnet
        label: Testnet Orderbook
"#,
            spec_version = SpecVersion::current()
        );

        let merged_config = merge_configstrings(dotrain.to_string(), config_text.to_string())
            .await
            .unwrap();

        // Verify networks were merged correctly
        assert_eq!(merged_config.networks.len(), 2);
        assert!(merged_config.networks.contains_key("mainnet"));
        assert!(merged_config.networks.contains_key("testnet"));

        // Verify subgraphs were merged correctly
        assert_eq!(merged_config.subgraphs.len(), 2);
        assert!(merged_config.subgraphs.contains_key("mainnet"));
        assert!(merged_config.subgraphs.contains_key("testnet"));

        // Verify orderbooks were merged correctly
        assert_eq!(merged_config.orderbooks.len(), 2);
        assert!(merged_config.orderbooks.contains_key("mainnetOrderbook"));
        assert!(merged_config.orderbooks.contains_key("testnetOrderbook"));

        // Verify raindex version was preserved
        assert_eq!(merged_config.version, "1".to_string());
    }

    #[tokio::test]
    async fn test_merge_configstrings_collision() {
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
subgraphs:
    mainnet: https://mainnet.subgraph
---
"#,
            spec_version = SpecVersion::current()
        );

        let config_text = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
subgraphs:
    mainnet: https://mainnet.subgraph
"#,
            spec_version = SpecVersion::current()
        );

        let err = merge_configstrings(dotrain.to_string(), config_text.to_string())
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::MergeError(MergeError::NetworkCollision(msg))
            if msg == "mainnet"
        ));
    }

    #[tokio::test]
    async fn test_merge_configstrings_invalid_config() {
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    mainnet: &mainnet
        rpcs:
            - https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
---
"#,
            spec_version = SpecVersion::current()
        );

        let config_text = format!(
            r#"
version: {spec_version}
networks:
    testnet: &testnet
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
"#,
            spec_version = SpecVersion::current()
        );

        let err = merge_configstrings(dotrain.to_string(), config_text.to_string())
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::ConfigSourceError(ConfigSourceError::YamlDeserializerError(msg))
            if msg.to_string().contains("missing field `rpcs`")
        ));
    }
}
