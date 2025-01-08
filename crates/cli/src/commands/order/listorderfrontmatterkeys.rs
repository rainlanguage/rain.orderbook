use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Debug, clap::ValueEnum, Clone, PartialEq)]
pub enum KeyType {
    Deployment,
    Scenario,
}

#[derive(Parser, Clone)]
pub struct ListOrderFrontmatterKeys {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'k', long, help = "Key Type", default_value = "deployment")]
    key_type: KeyType,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for ListOrderFrontmatterKeys {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };

        let order = DotrainOrder::new(dotrain, settings.map(|v| vec![v])).await?;

        let keys_string = match self.key_type {
            KeyType::Deployment => {
                let deployment_keys = order.dotrain_yaml.get_deployment_keys()?;
                deployment_keys
                    .iter()
                    .map(|key| key.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            }
            KeyType::Scenario => {
                let scenario_keys = order.dotrain_yaml.get_scenario_keys()?;
                scenario_keys
                    .iter()
                    .map(|key| key.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            }
        };

        output(&None, self.encoding.clone(), keys_string.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::str::FromStr;

    #[test]
    fn verify_cli() {
        ListOrderFrontmatterKeys::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let dotrain_file = PathBuf::from_str("./some/dotrain_file.rain").unwrap();
        let settings_file = PathBuf::from_str("./some/settings_file.rain").unwrap();
        let key_type = "deployment";
        let output_str = "binary";

        let cmd = ListOrderFrontmatterKeys::command();
        let result = cmd.get_matches_from(vec![
            "cmd",
            "-f",
            dotrain_file.to_str().unwrap(),
            "-c",
            settings_file.to_str().unwrap(),
            "-k",
            key_type,
            "-o",
            output_str,
        ]);
        assert_eq!(
            result.get_one::<PathBuf>("dotrain_file"),
            Some(&dotrain_file)
        );
        assert_eq!(
            result.get_one::<PathBuf>("settings_file"),
            Some(&settings_file)
        );
        assert_eq!(
            result.get_one::<KeyType>("key_type"),
            Some(&KeyType::Deployment)
        );
        assert_eq!(
            result.get_one::<SupportedOutputEncoding>("encoding"),
            Some(&SupportedOutputEncoding::Binary)
        );
    }

    fn get_test_dotrain(orderbook_key: &str) -> String {
        format!(
            "
networks:
    some-network:
        rpc: https://some-url.com
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    {}:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer
        bindings:
            some-bindings: 1
        scenarios: 
            some-other-scenario:
                bindings:
                    some-other-bindings: 1
            some-different-scenario:
                bindings:
                    some-different-bindings: 1

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        deployer: some-deployer

deployments:
    some-other-deployment:
        scenario: some-scenario.some-other-scenario
        order: some-order
    some-different-deployment:
        scenario: some-scenario.some-different-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;",
            orderbook_key
        )
    }

    #[tokio::test]
    async fn test_execute_deployment_key() {
        let dotrain = get_test_dotrain("some-orderbook");

        let dotrain_path = "./test_execute_deployment_key.rain";
        std::fs::write(dotrain_path, dotrain).unwrap();

        let keys = ListOrderFrontmatterKeys {
            dotrain_file: dotrain_path.into(),
            settings_file: None,
            key_type: KeyType::Deployment,
            encoding: SupportedOutputEncoding::Binary,
        };
        // should succeed without err
        keys.execute().await.unwrap();

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }

    #[tokio::test]
    async fn test_execute_scenario_key() {
        let dotrain = get_test_dotrain("some-orderbook");

        let dotrain_path = "./test_execute_scenario_key.rain";
        std::fs::write(dotrain_path, dotrain).unwrap();

        let keys = ListOrderFrontmatterKeys {
            dotrain_file: dotrain_path.into(),
            settings_file: None,
            key_type: KeyType::Scenario,
            encoding: SupportedOutputEncoding::Binary,
        };
        // should succeed without err
        keys.execute().await.unwrap();

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }
}
