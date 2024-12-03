mod tests {
    use super::super::orderbook::*;
    use crate::yaml::YamlError;
    use std::collections::HashMap;

    const VALID_YAML: &str = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: "1"
        label: "Ethereum Mainnet"
        network-id: "1"
        currency: "ETH"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/xyz
    secondary: https://api.thegraph.com/subgraphs/name/abc
metaboards:
    board1: https://meta.example.com/board1
    board2: https://meta.example.com/board2
orderbooks:
    book1:
        address: "0x1234567890abcdef"
        network: "mainnet"
        subgraph: "main"
        label: "Primary Orderbook"
tokens:
    weth:
        network: "mainnet"
        address: "0x2345678901abcdef"
        decimals: "18"
        label: "Wrapped Ether"
        symbol: "WETH"
deployers:
    mainnet:
        address: "0x3456789012abcdef"
        network: mainnet
        label: "Main Deployer"
accounts:
    admin: 0x4567890123abcdef
    user: 0x5678901234abcdef
sentry: true
"#;

    #[test]
    fn test_network() {
        let yaml = r#"
test: test
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc missing for network: \"mainnet\"".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("chain-id missing for network: \"mainnet\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        let network = config.networks.get("mainnet").unwrap();
        assert_eq!(network.rpc, "https://mainnet.infura.io");
        assert_eq!(network.chain_id, "1");
        assert_eq!(network.label, Some("Ethereum Mainnet".to_string()));
        assert_eq!(network.network_id, Some("1".to_string()));
        assert_eq!(network.currency, Some("ETH".to_string()));
    }

    #[test]
    fn test_subgraphs() {
        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: "1"
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field subgraphs".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: "1"
subgraphs:
    main:
        - one
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("subgraph value must be a string for key \"main\"".to_string())
        );
        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: "1"
subgraphs:
    main:
        - one: one
"#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("subgraph value must be a string for key \"main\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        assert_eq!(config.subgraphs.len(), 2);
        assert_eq!(
            config.subgraphs.get("mainnet").unwrap(),
            "https://api.thegraph.com/subgraphs/name/xyz"
        );
        assert_eq!(
            config.subgraphs.get("secondary").unwrap(),
            "https://api.thegraph.com/subgraphs/name/abc"
        );
    }

    #[test]
    fn test_metaboards() {
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field metaboards".to_string())
        );

        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1:
            - one
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key \"board1\"".to_string()
            )
        );
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1:
            - one: one
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key \"board1\"".to_string()
            )
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        assert_eq!(config.metaboards.len(), 2);
        assert_eq!(
            config.metaboards.get("board1").unwrap(),
            "https://meta.example.com/board1"
        );
        assert_eq!(
            config.metaboards.get("board2").unwrap(),
            "https://meta.example.com/board2"
        );
    }

    #[test]
    fn test_orderbooks() {
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field orderbooks".to_string())
        );

        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            network: "mainnet"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address missing for orderbook: \"book1\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        let orderbook = config.orderbooks.get("book1").unwrap();
        assert_eq!(orderbook.address, "0x1234567890abcdef");
        assert_eq!(orderbook.network, Some("mainnet".to_string()));
        assert_eq!(orderbook.subgraph, Some("main".to_string()));
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));
    }

    #[test]
    fn test_tokens() {
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field tokens".to_string())
        );

        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            address: "0x5678"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("network missing for token: \"weth\"".to_string())
        );

        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            network: "mainnet"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address missing for token: \"weth\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        let token = config.tokens.get("weth").unwrap();
        assert_eq!(token.network, "mainnet");
        assert_eq!(token.address, "0x2345678901abcdef");
        assert_eq!(token.decimals, Some("18".to_string()));
        assert_eq!(token.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token.symbol, Some("WETH".to_string()));
    }

    #[test]
    fn test_deployers() {
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            network: "mainnet"
            address: "0x5678"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field deployers".to_string())
        );

        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            network: "mainnet"
            address: "0x5678"
    deployers:
        main:
            network: "mainnet"
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address missing for deployer: \"main\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        let deployer = config.deployers.get("mainnet").unwrap();
        assert_eq!(deployer.address, "0x3456789012abcdef");
        assert_eq!(deployer.network, Some("mainnet".to_string()));
        assert_eq!(deployer.label, Some("Main Deployer".to_string()));
    }

    #[test]
    fn test_accounts() {
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            network: "mainnet"
            address: "0x5678"
    deployers:
        main:
            address: "0x9012"
    accounts:
        admin:
            - one
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("account value must be a string for key \"admin\"".to_string())
        );
        let yaml = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: "1"
    subgraphs:
        main: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        book1:
            address: "0x1234"
    tokens:
        weth:
            network: "mainnet"
            address: "0x5678"
    deployers:
        main:
            address: "0x9012"
    accounts:
        admin:
            - one: one
    "#;
        let error = OrderbookYaml::from_str(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("account value must be a string for key \"admin\"".to_string())
        );

        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        let accounts = config.accounts.unwrap();
        assert_eq!(accounts.get("admin").unwrap(), "0x4567890123abcdef");
        assert_eq!(accounts.get("user").unwrap(), "0x5678901234abcdef");
    }

    #[test]
    fn test_sentry() {
        let config = OrderbookYaml::from_str(VALID_YAML).unwrap();
        assert_eq!(config.sentry, Some("true".to_string()));
    }

    #[test]
    fn test_setter_methods() {
        let mut config = OrderbookYaml::default();

        let network = NetworkYaml {
            rpc: "https://rpc.example.com".to_string(),
            chain_id: "1".to_string(),
            label: Some("Test Network".to_string()),
            network_id: Some("1".to_string()),
            currency: Some("TEST".to_string()),
        };
        config.set_network("testnet".to_string(), network.clone());
        assert_eq!(config.networks.get("testnet").unwrap(), &network);

        config.set_subgraph(
            "test".to_string(),
            "https://subgraph.example.com".to_string(),
        );
        config.set_metaboard("test".to_string(), "https://meta.example.com".to_string());

        let orderbook = OrderbookEntryYaml {
            address: "0x1234".to_string(),
            network: Some("testnet".to_string()),
            subgraph: None,
            label: None,
        };
        config.set_orderbook("test".to_string(), orderbook);

        let token = TokenYaml {
            network: "testnet".to_string(),
            address: "0x5678".to_string(),
            decimals: None,
            label: None,
            symbol: None,
        };
        config.set_token("test".to_string(), token);

        let deployer = DeployerYaml {
            address: "0x9012".to_string(),
            network: None,
            label: None,
        };
        config.set_deployer("test".to_string(), deployer);

        let mut accounts = HashMap::new();
        accounts.insert("test".to_string(), "0x3456".to_string());
        config.set_accounts(accounts);

        config.set_sentry("https://sentry.example.com".to_string());

        // Verify all setters worked
        assert!(config.subgraphs.contains_key("test"));
        assert!(config.metaboards.contains_key("test"));
        assert!(config.orderbooks.contains_key("test"));
        assert!(config.tokens.contains_key("test"));
        assert!(config.deployers.contains_key("test"));
        assert!(config.accounts.is_some());
        assert!(config.sentry.is_some());
    }
}
