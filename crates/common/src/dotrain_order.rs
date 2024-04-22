use std::collections::HashMap;

use dotrain::{error::ComposeError, RainDocument};
use rain_orderbook_app_settings::{
    config_source::{ConfigSource, ConfigSourceError},
    merge::MergeError,
    Config, ParseConfigSourceError,
};
use thiserror::Error;

use crate::rainlang::compose_to_rainlang;

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

    #[error(transparent)]
    ComposeError(#[from] ComposeError),

    #[error(transparent)]
    MergeConfigError(#[from] MergeError),
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
        override_bindings: Vec<[String; 2]>,
    ) -> Result<String, DotrainOrderError> {
        let scenario = self
            .config
            .scenarios
            .get(&scenario)
            .ok_or_else(|| DotrainOrderError::ScenarioNotFound(scenario))?;

        let binds_map: HashMap<String, String> = override_bindings
            .iter()
            .map(|bind| (bind[0].clone(), bind[1].clone()))
            .collect();
        let merged_binds = scenario
            .bindings
            .clone()
            .into_iter()
            .chain(binds_map.into_iter())
            .collect::<HashMap<_, _>>();

        Ok(compose_to_rainlang(self.dotrain.clone(), merged_binds)?)
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
    async fn test_rainlang_from_binding() {
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
                bindings:
                    max-output: 10e18
                    io-ratio: 1e18
        ---
        #io-ratio !Ratio for calculating the output
        #max-output !Maximum output
        #calculate-io
        _ _: max-output io-ratio;
        #handle-io
        :;"#;

        let dotrain_order = DotrainOrder::new(dotrain.to_string(), None).await.unwrap();

        let override_bindings: Vec<[String; 2]> = vec![
            ["max-output".to_string(), "10000000000000000000".to_string()],
            ["io-ratio".to_string(), "1000000000000000000".to_string()],
        ];

        let rainlang = dotrain_order
            .compose_scenario_to_rainlang("polygon".to_string(), override_bindings)
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. calculate-io */ 
_ _: 10000000000000000000 1000000000000000000;

/* 1. handle-io */ 
:;"#
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
        let override_bindings: Vec<[String; 2]> = vec![];

        let rainlang = dotrain_order
            .compose_scenario_to_rainlang("polygon".to_string(), override_bindings)
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
}
