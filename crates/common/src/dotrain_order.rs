use alloy_ethers_typecast::transaction::{ReadableClient, ReadableClientError};
use alloy_primitives::{hex::encode, Address};
use dotrain::{error::ComposeError, RainDocument};
use rain_interpreter_parser::{ParserError, ParserV2};
use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{
    config_source::{ConfigSource, ConfigSourceError},
    merge::MergeError,
    Config, ParseConfigSourceError,
};
use std::collections::HashMap;
use thiserror::Error;
use typeshare::typeshare;

use crate::rainlang::compose_to_rainlang;

pub struct DotrainOrder {
    pub config: Config,
    pub dotrain: String,
}

#[typeshare]
pub type ScenariosAuthoringMeta = HashMap<String, Vec<AuthoringMetaV2>>;

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
        )?)
    }

    pub async fn get_pragmas_for_scenario(
        &self,
        scenario: String,
    ) -> Result<Vec<Address>, DotrainOrderError> {
        let deployer = &self
            .config
            .scenarios
            .get(&scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario.clone()))?
            .deployer;
        let parser: ParserV2 = deployer.address.into();
        let rainlang = self.compose_scenario_to_rainlang(scenario).await?;
        println!("rainlang: {:?}", rainlang.clone());
        println!("rainlang: {:?}", encode(rainlang.as_bytes()));
        let client = ReadableClient::new_from_url(deployer.network.rpc.clone().to_string())?;
        let mut pragmas = parser.parse_pragma_text(&*rainlang, client).await?;
        pragmas.push(deployer.address);

        Ok(pragmas)
    }

    pub async fn get_authoring_meta_v2_for_scenario(
        &self,
        scenario: String,
    ) -> Result<Vec<AuthoringMetaV2>, DotrainOrderError> {
        let rpc = self
            .config
            .scenarios
            .get(&scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario.clone()))?
            .deployer
            .network
            .rpc
            .clone();

        let network_name = &self
            .config
            .scenarios
            .get(&scenario)
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

        let pragmas = self.get_pragmas_for_scenario(scenario).await?;

        let mut authoring_metas = Vec::new();

        for pragma in pragmas {
            let authoring_meta_v2 =
                AuthoringMetaV2::fetch_for_contract(pragma, rpc.to_string(), metaboard.to_string())
                    .await?;
            authoring_metas.push(authoring_meta_v2);
        }

        Ok(authoring_metas)
    }

    pub async fn get_authoring_metas_for_all_scenarios(
        &self,
    ) -> Result<ScenariosAuthoringMeta, DotrainOrderError> {
        let mut authoring_metas = HashMap::new();

        for scenario in self.config.scenarios.keys() {
            let authoring_meta_v2 = self
                .get_authoring_meta_v2_for_scenario(scenario.clone())
                .await?;
            authoring_metas.insert(scenario.clone(), authoring_meta_v2);
        }

        Ok(authoring_metas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_parse() {
        let dotrain = r#"
networks:
    polygon: 
        rpc: https://rpc.ankr.com/polygon 
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
:;"#;

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        assert_eq!(
            dotrain_order.config.networks.get("polygon").unwrap().rpc,
            "https://rpc.ankr.com/polygon".parse().unwrap()
        );
    }

    #[tokio::test]
    async fn test_rainlang_from_scenario() {
        let dotrain = r#"
networks:
    polygon: 
        rpc: https://rpc.ankr.com/polygon 
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
:;"#;

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
    async fn test_config_merge() {
        let dotrain = r#"
networks: 
  polygon: 
    rpc: https://rpc.ankr.com/polygon 
    chain-id: 137 
    network-id: 137 
    currency: MATIC
---
#calculate-io
_ _: 00;

#handle-io
:;"#;

        let settings = r#"
networks:
    mainnet: 
        rpc: https://1rpc.io/eth 
        chain-id: 1 
        network-id: 1 
        currency: ETH"#;

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
            "https://1rpc.io/eth".parse().unwrap()
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
            .get_pragmas_for_scenario("sepolia".to_string())
            .await
            .unwrap();

        println!("{:?}", pragmas);

        assert_eq!(pragmas.len(), 2);
    }

    #[tokio::test]
    async fn test_get_authoring_meta_for_all_scenarios() {
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
    sepolia: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/test-mb-sepolia/0.0.1/gn
---
#calculate-io
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL,
        );

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let authoring_metas = dotrain_order
            .get_authoring_metas_for_all_scenarios()
            .await
            .unwrap();

        println!("{:?}", authoring_metas);
    }
}
