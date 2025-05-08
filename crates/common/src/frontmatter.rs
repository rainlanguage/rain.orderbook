use dotrain::RainDocument;
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{Config, ParseConfigError};

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn parse_frontmatter(dotrain: String, validate: bool) -> Result<Config, ParseConfigError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Config::try_from_yaml(vec![frontmatter.to_string()], validate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TEST_DOTRAIN;
    use alloy::primitives::{Address, U256};
    use rain_orderbook_app_settings::plot_source::{
        BinXTransformCfg, DotOptionsCfg, HexBinTransformCfg, LineOptionsCfg, MarkCfg,
        RectYOptionsCfg, TransformCfg,
    };
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_parse_networks() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.networks.len(), 2);
        let mainnet_network = config.networks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(mainnet_network.chain_id, 1);
        assert!(mainnet_network.label.is_none());
        assert!(mainnet_network.network_id.is_none());
        assert!(mainnet_network.currency.is_none());
        let testnet_network = config.networks.get("testnet").unwrap();
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

        assert_eq!(config.subgraphs.len(), 2);
        let mainnet_subgraph = config.subgraphs.get("mainnet").unwrap();
        assert_eq!(
            mainnet_subgraph,
            &Url::parse("https://mainnet-subgraph.com").unwrap()
        );
        let testnet_subgraph = config.subgraphs.get("testnet").unwrap();
        assert_eq!(
            testnet_subgraph,
            &Url::parse("https://testnet-subgraph.com").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_metaboards() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.metaboards.len(), 2);
        let mainnet_metaboard = config.metaboards.get("mainnet").unwrap();
        assert_eq!(
            mainnet_metaboard,
            &Url::parse("https://mainnet-metaboard.com").unwrap()
        );
        let testnet_metaboard = config.metaboards.get("testnet").unwrap();
        assert_eq!(
            testnet_metaboard,
            &Url::parse("https://testnet-metaboard.com").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_orderbooks() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.orderbooks.len(), 2);
        let mainnet_orderbook = config.orderbooks.get("mainnet").unwrap();
        assert_eq!(
            mainnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        let testnet_orderbook = config.orderbooks.get("testnet").unwrap();
        assert_eq!(
            testnet_orderbook.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_tokens() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.tokens.len(), 2);
        let token1 = config.tokens.get("token1").unwrap();
        assert_eq!(
            token1.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token1.decimals, Some(18));
        assert_eq!(token1.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token1.symbol, Some("WETH".to_string()));
        let token2 = config.tokens.get("token2").unwrap();
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

        assert_eq!(config.deployers.len(), 2);
        let deployer_scenario1 = config.deployers.get("scenario1").unwrap();
        assert_eq!(
            deployer_scenario1.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        let deployer2 = config.deployers.get("deployer2").unwrap();
        assert_eq!(
            deployer2.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }

    #[tokio::test]
    async fn test_parse_orders() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.orders.len(), 1);
        let order1 = config.orders.get("order1").unwrap();
        assert_eq!(order1.inputs.len(), 1);
        let order1_input = &order1.inputs[0];
        assert_eq!(order1_input.token, "token1");
        assert_eq!(order1_input.vault_id, Some(U256::from(1)));
        assert_eq!(order1.outputs.len(), 1);
        let order1_output = &order1.outputs[0];
        assert_eq!(order1_output.token, "token2");
        assert_eq!(order1_output.vault_id, Some(U256::from(2)));
    }

    #[tokio::test]
    async fn test_parse_scenarios() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.scenarios.len(), 1);
        let scenario1 = config.scenarios.get("scenario1").unwrap();
        assert_eq!(scenario1.bindings.len(), 1);
        assert_eq!(scenario1.bindings.get("key1").unwrap(), "value1");
        assert!(scenario1.runs.is_none());
        assert!(scenario1.blocks.is_none());
        let scenario1_scenarios = scenario1.scenarios.as_ref().unwrap();
        let scenario1_scenario2 = scenario1_scenarios.get("scenario2").unwrap();
        assert_eq!(scenario1_scenario2.bindings.len(), 1);
        assert_eq!(scenario1_scenario2.bindings.get("key2").unwrap(), "value2");
        assert_eq!(scenario1_scenario2.runs.unwrap(), 10);
        assert!(scenario1_scenario2.blocks.is_none());
    }

    #[tokio::test]
    async fn test_parse_deployments() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.deployments.len(), 2);
        let deployment1 = config.deployments.get("deployment1").unwrap();
        assert_eq!(deployment1.order, "order1");
        assert_eq!(deployment1.scenario, "scenario1.scenario2");
        let deployment2 = config.deployments.get("deployment2").unwrap();
        assert_eq!(deployment2.order, "order1");
        assert_eq!(deployment2.scenario, "scenario1");
    }

    #[tokio::test]
    async fn test_parse_charts() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert_eq!(config.charts.len(), 1);
        let chart1 = config.charts.get("chart1").unwrap();
        assert!(chart1.plots.is_some());
        assert!(chart1.metrics.is_none());
        let plots = chart1.plots.as_ref().unwrap();
        assert_eq!(plots.len(), 1);
        let plot1 = plots.get("plot1").unwrap();
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

        assert!(config.gui.is_some());
        let gui = config.gui.as_ref().unwrap();
        assert_eq!(gui.name, "Test gui");
        assert_eq!(gui.description, "Test description");
        assert_eq!(gui.deployments.len(), 1);
        let gui_deployment1 = gui.deployments.get("deployment1").unwrap();
        assert_eq!(gui_deployment1.name, "Test deployment");
        assert_eq!(gui_deployment1.description, "Test description");
        assert_eq!(gui_deployment1.deposits.len(), 1);
        let deposit1 = &gui_deployment1.deposits[0];
        assert_eq!(deposit1.token, "token1");
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

        assert_eq!(config.sentry, Some(true));
        let sentry = config.sentry.unwrap();
        assert!(sentry);
    }

    #[tokio::test]
    async fn test_parse_raindex_version() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert!(config.raindex_version.is_some());
        let raindex_version = config.raindex_version.as_ref().unwrap();
        assert_eq!(raindex_version, "123");
    }

    #[tokio::test]
    async fn test_parse_accounts() {
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert!(config.accounts.is_some());
        let accounts = config.accounts.as_ref().unwrap();
        let account1 = accounts.get("account1").unwrap();
        assert_eq!(account1, "0x0000000000000000000000000000000000000001");
        let account2 = accounts.get("account2").unwrap();
        assert_eq!(account2, "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    }

    #[tokio::test]
    async fn test_parse_overall_counts() {
        // This test checks the top-level counts that were previously checked
        // at the beginning of the original test.
        let config = parse_frontmatter(TEST_DOTRAIN.to_string()).await.unwrap();

        assert!(config.raindex_version.is_some());
        assert_eq!(config.networks.len(), 2);
        assert_eq!(config.subgraphs.len(), 2);
        assert_eq!(config.metaboards.len(), 2);
        assert_eq!(config.orderbooks.len(), 2);
        assert_eq!(config.tokens.len(), 2);
        assert_eq!(config.deployers.len(), 2);
        assert_eq!(config.sentry, Some(true));
        assert!(config.accounts.is_some());
        assert_eq!(config.orders.len(), 1);
        assert_eq!(config.scenarios.len(), 1);
        assert_eq!(config.deployments.len(), 2);
        assert_eq!(config.charts.len(), 1);
        assert!(config.gui.is_some());
    }
}
