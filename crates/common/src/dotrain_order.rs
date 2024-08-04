use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::{ReadableClient, ReadableClientError};
use dotrain::{error::ComposeError, RainDocument};
use rain_interpreter_parser::{ParserError, ParserV2};
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{
    config_source::{ConfigSource, ConfigSourceError},
    merge::MergeError,
    Config, ParseConfigSourceError,
};
use thiserror::Error;

use crate::{
    add_order::{ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS, ORDERBOOK_ORDER_ENTRYPOINTS},
    rainlang::compose_to_rainlang,
};

#[derive(Clone)]
pub struct DotrainOrder {
    pub config: Config,
    pub dotrain: String,
}

#[derive(Error, Debug)]
pub enum DotrainOrderError {
    #[error(transparent)]
    ConfigSourceError(#[from] ConfigSourceError),

    #[error(transparent)]
    ParseConfigSourceError(#[from] ParseConfigSourceError),

    #[error("Scenario {0} not found")]
    ScenarioNotFound(String),

    #[error("Metaboard {0} not found")]
    MetaboardNotFound(String),

    #[error(transparent)]
    ComposeError(#[from] ComposeError),

    #[error(transparent)]
    MergeConfigError(#[from] MergeError),

    #[error(transparent)]
    AuthoringMetaV2Error(#[from] AuthoringMetaV2Error),

    #[error(transparent)]
    FetchAuthoringMetaV2WordError(#[from] FetchAuthoringMetaV2WordError),

    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),

    #[error(transparent)]
    ParserError(#[from] ParserError),
}

impl DotrainOrder {
    pub async fn new(dotrain: String, config: Option<String>) -> Result<Self, DotrainOrderError> {
        match config {
            Some(config) => {
                let config_string = ConfigSource::try_from_string(config).await?;
                let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
                let mut frontmatter_config =
                    ConfigSource::try_from_string(frontmatter.to_string()).await?;
                frontmatter_config.merge(config_string)?;
                Ok(Self {
                    dotrain,
                    config: frontmatter_config.try_into()?,
                })
            }
            None => {
                let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
                let config_string = ConfigSource::try_from_string(frontmatter.to_string()).await?;
                let config: Config = config_string.try_into()?;
                Ok(Self { dotrain, config })
            }
        }
    }

    pub async fn compose_scenario_to_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        let scenario = self
            .config
            .scenarios
            .get(&scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario))?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )?)
    }

    pub async fn compose_scenario_to_post_task_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        let scenario = self
            .config
            .scenarios
            .get(&scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario))?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS,
        )?)
    }

    pub async fn get_pragmas_for_scenario(
        &self,
        scenario: &String,
    ) -> Result<Vec<Address>, DotrainOrderError> {
        let deployer = &self
            .config
            .scenarios
            .get(scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario.clone()))?
            .deployer;
        let parser: ParserV2 = deployer.address.into();
        let rainlang = self.compose_scenario_to_rainlang(scenario.clone()).await?;

        let client = ReadableClient::new_from_url(deployer.network.rpc.clone().to_string())?;
        let pragmas = parser.parse_pragma_text(&rainlang, client).await?;
        Ok(pragmas)
    }

    pub async fn get_authoring_meta_v2_for_scenario_pragma(
        &self,
        scenario: &String,
        pragma: &Address,
    ) -> Result<AuthoringMetaV2, DotrainOrderError> {
        let rpc = self
            .config
            .scenarios
            .get(scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario.clone()))?
            .deployer
            .network
            .rpc
            .clone();

        let network_name = &self
            .config
            .scenarios
            .get(scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario.clone()))?
            .deployer
            .network
            .name;

        let metaboard = self
            .config
            .metaboards
            .get(network_name)
            .ok_or_else(|| DotrainOrderError::MetaboardNotFound(network_name.clone()))?
            .clone();

        let authoring_meta_v2 =
            AuthoringMetaV2::fetch_for_contract(*pragma, rpc.to_string(), metaboard.to_string())
                .await?;

        Ok(authoring_meta_v2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_parse() {
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        assert_eq!(
            dotrain_order.config.networks.get("polygon").unwrap().rpc,
            rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL
                .parse()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_rainlang_from_scenario() {
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let rainlang = dotrain_order
            .compose_scenario_to_rainlang("polygon".to_string())
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;"#
        );
    }

    #[tokio::test]
    async fn test_rainlang_post_from_scenario() {
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
_ _: 1 2;
"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let rainlang = dotrain_order
            .compose_scenario_to_post_task_rainlang("polygon".to_string())
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. post-add-order */ 
_ _: 1 2;
:;"#
        );
    }

    #[tokio::test]
    async fn test_config_merge() {
        let dotrain = format!(
            r#"
networks:
  polygon:
    rpc: {rpc_url}
    chain-id: 137
    network-id: 137
    currency: MATIC
---
#calculate-io
_ _: 00;

#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL
        );

        let settings = format!(
            r#"
networks:
    mainnet:
        rpc: {rpc_url}
        chain-id: 1
        network-id: 1
        currency: ETH"#,
            rpc_url = rain_orderbook_env::CI_RPC_URL_ETHEREUM_FORK
        );

        let merged_dotrain_order =
            DotrainOrder::new(dotrain.to_string(), Some(settings.to_string()))
                .await
                .unwrap();

        assert_eq!(
            merged_dotrain_order
                .config
                .networks
                .get("mainnet")
                .unwrap()
                .rpc,
            rain_orderbook_env::CI_RPC_URL_ETHEREUM_FORK
                .parse()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_pragmas_for_scenario() {
        let dotrain = format!(
            r#"
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 0
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
scenarios:
    sepolia:
---
#calculate-io
using-words-from 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let pragmas = dotrain_order
            .get_pragmas_for_scenario(&"sepolia".to_string())
            .await
            .unwrap();

        assert_eq!(pragmas.len(), 1);
    }

    #[tokio::test]
    async fn test_get_get_authoring_meta_v2_for_scenario_pragma() {
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
    scenarios:
        sepolia:
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from 0x2382e861cF4F47578aC29B50944b3b445577aF74
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
            metaboard_url = rain_orderbook_env::CI_SEPOLIA_METABOARD_URL,
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let pragmas = dotrain_order
            .get_pragmas_for_scenario(&"sepolia".to_string())
            .await
            .unwrap();

        println!("{:?}", pragmas);

        let authoring_meta_v2 = dotrain_order
            .get_authoring_meta_v2_for_scenario_pragma(&"sepolia".to_string(), &pragmas[0])
            .await
            .unwrap();

        println!("{:?}", authoring_meta_v2);
    }
}
