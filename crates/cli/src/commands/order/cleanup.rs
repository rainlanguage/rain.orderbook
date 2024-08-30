use crate::execute::Execute;
use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

/// Generate a new .rain with unused frontmatter cleaned, ie frontmatter will only include the
/// specified deployments (and their related fields) from a given .rain and an optional setting.yml
#[derive(Parser, Clone)]
pub struct Cleanup {
    /// Path to the .rain file
    #[arg(short = 'f', long, value_name = "PATH")]
    dotrain_file: PathBuf,

    /// Path to the settings yaml file
    #[arg(short = 'c', long, value_name = "PATH")]
    settings_file: Option<PathBuf>,

    /// List of deployment keys to include in the output .rain frontmatter
    #[arg(short = 'e', long, value_name = "DEPLOYMENT", num_args = 1..)]
    deployments: Vec<String>,

    /// Optional output file path to write the result into
    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Print the result on console (send result to std out)
    #[arg(long, action = ArgAction::SetTrue)]
    pub stdout: bool,
}

impl Execute for Cleanup {
    async fn execute(&self) -> Result<()> {
        // read inpput files
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };

        // generate new dotrain order instance with cleaned up frontmatter
        let order = DotrainOrder::new_with_deployments_frontmatter(
            dotrain,
            settings,
            &self
                .deployments
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>(),
        )
        .await?;

        // handle output
        if let Some(output) = &self.output {
            std::fs::write(output, &order.dotrain)?;
        }
        if self.stdout {
            println!("{}", order.dotrain);
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
        Cleanup::command().debug_assert();
    }

    #[tokio::test]
    async fn test_execute_happy() {
        let setting = r#"
networks:
    some-network:
        rpc: https://abcd.com
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
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
"#;
        let dotrain = r#"
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
        scenarios: 
            child-scenario:
                bindings:
                    key1: value1

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token1
              vault-id: 1
        deployer: some-deployer
        orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario.child-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#;

        let dotrain_path = "./test_dotrain_cleanup.rain";
        let settings_path = "./test_settings_cleanup.yml";
        std::fs::write(dotrain_path, dotrain).unwrap();
        std::fs::write(settings_path, setting).unwrap();

        let cleanup = Cleanup {
            dotrain_file: dotrain_path.into(),
            settings_file: Some(settings_path.into()),
            deployments: vec!["some-deployment".to_string()],
            output: None,
            stdout: true,
        };

        assert!(cleanup.execute().await.is_ok());

        std::fs::remove_file(dotrain_path).unwrap();
        std::fs::remove_file(settings_path).unwrap();
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let cleanup = Cleanup {
            dotrain_file: "./bad-path/test.rain".into(),
            settings_file: None,
            deployments: vec!["some-deployment".to_string()],
            output: None,
            stdout: true,
        };

        assert!(cleanup.execute().await.is_err());
    }
}
