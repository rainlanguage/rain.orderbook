use dotrain::RainDocument;
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{config::Config, ParseConfigError};

pub async fn parse_frontmatter(dotrain: String) -> Result<Config, ParseConfigError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Config::try_from_yaml(vec![frontmatter.to_string()], false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TEST_DOTRAIN;
    use alloy::primitives::{Address, U256};
    use rain_orderbook_app_settings::{
        plot_source::{
            BinXTransformCfg, DotOptionsCfg, HexBinTransformCfg, LineOptionsCfg, MarkCfg,
            RectYOptionsCfg, TransformCfg,
        },
        spec_version::SpecVersion,
    };
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_parse_networks() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_networks().len(), 2);
        let networks = config.get_networks();
        let mainnet_network = networks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(mainnet_network.chain_id, 1);
        assert!(mainnet_network.label.is_none());
        assert!(mainnet_network.network_id.is_none());
        assert!(mainnet_network.currency.is_none());
        let testnet_network = networks.get("testnet").unwrap();
        assert_eq!(
            testnet_network.rpc,
            Url::parse("https://testnet.infura.io").unwrap()
        );
        assert_eq!(testnet_network.chain_id, 1337);
        assert!(testnet_network.label.is_none());
        assert!(testnet_network.network_id.is_none());
        assert!(testnet_network.currency.is_none());
    }

    #[tokio::test]
    async fn test_parse_subgraphs() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_subgraphs().len(), 2);
        let subgraphs = config.get_subgraphs();
        let mainnet_subgraph = subgraphs.get("mainnet").unwrap();
        assert_eq!(
            mainnet_subgraph.url,
            Url::parse("https://mainnet-subgraph.com").unwrap()
        );
        let testnet_subgraph = subgraphs.get("testnet").unwrap();
        assert_eq!(
            testnet_subgraph.url,
            Url::parse("https://testnet-subgraph.com").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_metaboards() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_metaboards().len(), 2);
        let metaboards = config.get_metaboards();
        let mainnet_metaboard = metaboards.get("mainnet").unwrap();
        assert_eq!(
            mainnet_metaboard.as_ref(),
            &Url::parse("https://mainnet-metaboard.com").unwrap()
        );
        let testnet_metaboard = metaboards.get("testnet").unwrap();
        assert_eq!(
            testnet_metaboard.as_ref(),
            &Url::parse("https://testnet-metaboard.com").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_orderbooks() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_orderbooks().len(), 2);
        let orderbooks = config.get_orderbooks();
        let mainnet_orderbook = orderbooks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        let testnet_orderbook = orderbooks.get("testnet").unwrap();
        assert_eq!(
            testnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_tokens() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_tokens().len(), 2);
        let tokens = config.get_tokens();
        let token1 = tokens.get("token1").unwrap();
        assert_eq!(
            token1.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token1.decimals, Some(18));
        assert_eq!(token1.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token1.symbol, Some("WETH".to_string()));
        let token2 = tokens.get("token2").unwrap();
        assert_eq!(
            token2.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token2.decimals, Some(6));
        assert_eq!(token2.label, Some("USD Coin".to_string()));
        assert_eq!(token2.symbol, Some("USDC".to_string()));
    }

    #[tokio::test]
    async fn test_parse_deployers() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_deployers().len(), 2);
        let deployers = config.get_deployers();
        let deployer_scenario1 = deployers.get("scenario1").unwrap();
        assert_eq!(
            deployer_scenario1.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        let deployer2 = deployers.get("deployer2").unwrap();
        assert_eq!(
            deployer2.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_orders() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_orders().len(), 1);
        let orders = config.get_orders();
        let order1 = orders.get("order1").unwrap();
        assert_eq!(order1.inputs.len(), 1);
        let order1_input = &order1.inputs[0];
        assert_eq!(order1_input.token.as_ref().unwrap().key, "token1");
        assert_eq!(order1_input.vault_id, Some(U256::from(1)));
        assert_eq!(order1.outputs.len(), 1);
        let order1_output = &order1.outputs[0];
        assert_eq!(order1_output.token.as_ref().unwrap().key, "token2");
        assert_eq!(order1_output.vault_id, Some(U256::from(2)));
    }

    #[tokio::test]
    async fn test_parse_scenarios() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_scenarios().len(), 2);
        let scenarios = config.get_scenarios();
        let scenario1 = scenarios.get("scenario1").unwrap();
        assert_eq!(scenario1.bindings.len(), 1);
        assert_eq!(scenario1.bindings.get("key1").unwrap(), "10");
        assert!(scenario1.runs.is_none());
        assert!(scenario1.blocks.is_none());
        let scenario2 = scenarios.get("scenario1.scenario2").unwrap();
        assert_eq!(scenario2.bindings.len(), 2);
        assert_eq!(scenario2.bindings.get("key2").unwrap(), "20");
        assert_eq!(scenario2.runs.unwrap(), 10);
        assert!(scenario2.blocks.is_none());
    }

    #[tokio::test]
    async fn test_parse_deployments() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_deployments().len(), 2);
        let deployments = config.get_deployments();
        let deployment1 = deployments.get("deployment1").unwrap();
        assert_eq!(deployment1.order.key, "order1");
        assert_eq!(deployment1.scenario.key, "scenario1.scenario2");
        let deployment2 = deployments.get("deployment2").unwrap();
        assert_eq!(deployment2.order.key, "order1");
        assert_eq!(deployment2.scenario.key, "scenario1");
    }

    #[tokio::test]
    async fn test_parse_charts() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.get_charts().len(), 1);
        let charts = config.get_charts();
        let chart1 = charts.get("chart1").unwrap();
        assert!(chart1.plots.is_some());
        assert!(chart1.metrics.is_none());
        let plots = chart1.plots.as_ref().unwrap();
        assert_eq!(plots.len(), 1);
        let plot1 = plots[0].clone();
        assert_eq!(plot1.title, Some("Test title".to_string()));
        assert_eq!(plot1.subtitle, Some("Test subtitle".to_string()));
        assert_eq!(plot1.marks.len(), 3);
        match &plot1.marks[0] {
            MarkCfg::Dot(DotOptionsCfg {
                x,
                y,
                r,
                fill,
                stroke,
                transform,
            }) => {
                assert_eq!(x.as_deref(), Some("1"));
                assert_eq!(y.as_deref(), Some("2"));
                assert_eq!(r, &Some(3));
                assert_eq!(fill.as_deref(), Some("red"));
                assert_eq!(stroke.as_deref(), Some("blue"));
                assert!(transform.is_some());
                match transform.as_ref().unwrap() {
                    TransformCfg::HexBin(HexBinTransformCfg { outputs, options }) => {
                        assert_eq!(outputs.x.as_deref(), Some("1"));
                        assert_eq!(outputs.y.as_deref(), Some("2"));
                        assert_eq!(outputs.r, Some(3));
                        assert_eq!(outputs.z.as_deref(), Some("4"));
                        assert_eq!(outputs.stroke.as_deref(), Some("green"));
                        assert_eq!(outputs.fill.as_deref(), Some("blue"));
                        assert_eq!(options.x.as_deref(), Some("1"));
                        assert_eq!(options.y.as_deref(), Some("2"));
                        assert_eq!(options.bin_width, Some(10));
                    }
                    _ => panic!("Incorrect transform type for mark 0"),
                }
            }
            _ => panic!("Incorrect mark type for mark 0"),
        }
        match &plot1.marks[1] {
            MarkCfg::Line(LineOptionsCfg { transform, .. }) => {
                assert!(transform.is_some());
                match transform.as_ref().unwrap() {
                    TransformCfg::BinX(BinXTransformCfg { outputs, options }) => {
                        assert_eq!(outputs.x.as_deref(), Some("1"));
                        // other outputs not specified, should be None
                        assert!(outputs.y.is_none());
                        assert!(outputs.r.is_none());
                        assert!(outputs.z.is_none());
                        assert!(outputs.stroke.is_none());
                        assert!(outputs.fill.is_none());
                        // options x not specified, should be None
                        assert!(options.x.is_none());
                        assert_eq!(options.thresholds, Some(10));
                    }
                    _ => panic!("Incorrect transform type for mark 1"),
                }
            }
            _ => panic!("Incorrect mark type for mark 1"),
        }
        match &plot1.marks[2] {
            MarkCfg::RectY(RectYOptionsCfg {
                x0,
                x1,
                y0,
                y1,
                transform,
            }) => {
                assert_eq!(x0.as_deref(), Some("1"));
                assert_eq!(x1.as_deref(), Some("2"));
                assert_eq!(y0.as_deref(), Some("3"));
                assert_eq!(y1.as_deref(), Some("4"));
                assert!(transform.is_none());
            }
            _ => panic!("Incorrect mark type for mark 2"),
        }
        assert!(plot1.x.is_some());
        let axis_x = plot1.x.as_ref().unwrap();
        assert_eq!(axis_x.label, Some("Test x label".to_string()));
        assert_eq!(axis_x.anchor, Some("start".to_string()));
        assert_eq!(axis_x.label_anchor, Some("start".to_string()));
        assert_eq!(axis_x.label_arrow, Some("none".to_string()));
        assert!(plot1.y.is_some());
        let axis_y = plot1.y.as_ref().unwrap();
        assert_eq!(axis_y.label, Some("Test y label".to_string()));
        assert_eq!(axis_y.anchor, Some("start".to_string()));
        assert_eq!(axis_y.label_anchor, Some("start".to_string()));
        assert_eq!(axis_y.label_arrow, Some("none".to_string()));
        assert_eq!(plot1.margin, Some(10));
        assert_eq!(plot1.margin_left, Some(20));
        assert_eq!(plot1.margin_right, Some(30));
        assert_eq!(plot1.margin_top, Some(40));
        assert_eq!(plot1.margin_bottom, Some(50));
        assert_eq!(plot1.inset, Some(60));
    }

    #[tokio::test]
    async fn test_parse_gui() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert!(config.get_gui().is_some());
        let gui = config.get_gui().as_ref().unwrap();
        assert_eq!(gui.name, "Test gui");
        assert_eq!(gui.description, "Test description");
        assert_eq!(gui.deployments.len(), 1);
        let gui_deployment1 = gui.deployments.get("deployment1").unwrap();
        assert_eq!(gui_deployment1.name, "Test deployment");
        assert_eq!(gui_deployment1.description, "Test description");
        assert_eq!(gui_deployment1.deposits.len(), 1);
        let deposit1 = &gui_deployment1.deposits[0];
        assert_eq!(deposit1.token.as_ref().unwrap().key, "token1");
        assert_eq!(
            deposit1.presets,
            Some(vec!["100".to_string(), "2000".to_string()])
        );
        assert_eq!(gui_deployment1.fields.len(), 1);
        let field1 = &gui_deployment1.fields[0];
        assert_eq!(field1.binding, "key1");
        assert_eq!(field1.name, "Binding test");
        assert!(field1.description.is_none());
        assert!(field1.presets.is_some());
        let field1_presets = field1.presets.as_ref().unwrap();
        assert_eq!(field1_presets.len(), 1);
        assert!(field1_presets[0].name.is_none());
        assert_eq!(field1_presets[0].value, "value2");
        assert!(field1.default.is_none());
        assert!(field1.show_custom_field.is_none());
        assert!(gui_deployment1.select_tokens.is_some());
        let select_tokens = gui_deployment1.select_tokens.as_ref().unwrap();
        assert_eq!(select_tokens.len(), 1);
        let select_token1 = &select_tokens[0];
        assert_eq!(select_token1.key, "token2");
        assert_eq!(select_token1.name, Some("Test token".to_string()));
        assert_eq!(
            select_token1.description,
            Some("Test description".to_string())
        );
    }

    #[tokio::test]
    async fn test_parse_sentry() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();
        assert_eq!(config.get_sentry(), &Some(true));
    }

    #[tokio::test]
    async fn test_parse_spec_version() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();
        assert_eq!(config.get_version(), &SpecVersion::current().to_string());
    }

    #[tokio::test]
    async fn test_parse_accounts() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        let accounts = config.get_accounts();
        let account1 = accounts.get("account1").unwrap();
        assert_eq!(
            account1.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        let account2 = accounts.get("account2").unwrap();
        assert_eq!(
            account2.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }
}
