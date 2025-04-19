use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use anyhow::{anyhow, Result};
use clap::{ArgAction, Args};
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::frontmatter::parse_frontmatter;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::GH_COMMIT_SHA;
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderAddArgs {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,

    /// Do NOT broadcast the transaction to the network, only simulate the transaction
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_broadcast: bool,

    /// Skips checking the `raindex-version` against this cli app current version
    #[arg(long, action = ArgAction::SetTrue)]
    pub skip_version_check: bool,
}

impl CliOrderAddArgs {
    async fn to_add_order_args(&self) -> Result<AddOrderArgs> {
        let text = read_to_string(&self.dotrain_file).map_err(|e| anyhow!(e))?;
        let config = parse_frontmatter(text.clone(), true)?;

        if !self.skip_version_check {
            if let Some(ver) = config.get_raindex_version() {
                if ver.to_ascii_lowercase() != GH_COMMIT_SHA.to_ascii_lowercase() {
                    return Err(anyhow!(format!(
                        "mismatch raindex version: expected: {}, got: {}",
                        GH_COMMIT_SHA, ver
                    )));
                }
            } else {
                return Err(anyhow!(
                    "missing 'raindex-version' field in .rain file frontmatter"
                ));
            }
        }

        let config_deployment = config
            .get_deployments()
            .get(&self.deployment)
            .ok_or(anyhow!("specified deployment is undefined!"))?;

        Ok(
            AddOrderArgs::new_from_deployment(text.clone(), config_deployment.deref().clone())
                .await?,
        )
    }
}

impl Execute for CliOrderAddArgs {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.clone().to_add_order_args().await?;
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        info!("----- Simulating Transaction -----");
        add_order_args
            .simulate_execute(tx_args.clone(), None)
            .await?;
        info!("----- Finished Simulation Successfully -----");

        if !self.no_broadcast {
            info!("----- Add Order -----");
            add_order_args
                .execute(tx_args, |status| {
                    display_write_transaction_status(status);
                })
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use rain_orderbook_bindings::IOrderBookV4::IO;
    use std::{collections::HashMap, str::FromStr};

    #[tokio::test]
    async fn test_to_add_order_args() {
        let dotrain = get_dotrain(GH_COMMIT_SHA);

        let dotrain_path = "./test_dotrain_add_order.rain";
        std::fs::write(dotrain_path, dotrain).unwrap();

        let cli_order_add_args = CliOrderAddArgs {
            no_broadcast: false,
            dotrain_file: dotrain_path.into(),
            deployment: "some-deployment".to_string(),
            skip_version_check: false,
            transaction_args: CliTransactionArgs {
                orderbook_address: Address::random(),
                derivation_index: None,
                chain_id: Some(123),
                rpc_url: "https://some-rpc.com".to_string(),
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                gas_fee_speed: None,
            },
        };

        let result = cli_order_add_args.to_add_order_args().await.unwrap();
        let expected = AddOrderArgs {
            dotrain: get_dotrain(GH_COMMIT_SHA),
            inputs: vec![IO {
                token: Address::from_str("0xc2132d05d31c914a87c6611c10748aeb04b58e8f").unwrap(),
                decimals: 6,
                vaultId: U256::from(1),
            }],
            outputs: vec![IO {
                token: Address::from_str("0x8f3cf7ad23cd3cadbd9735aff958023239c6a063").unwrap(),
                decimals: 18,
                vaultId: U256::from(1),
            }],
            deployer: Address::from_str("0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba").unwrap(),
            bindings: HashMap::new(),
        };
        assert_eq!(result, expected);

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }

    #[tokio::test]
    async fn test_to_add_order_args_version_check_unhappy() {
        let dotrain = get_dotrain("1234");

        let dotrain_path = "./test_dotrain_add_order2.rain";
        std::fs::write(dotrain_path, dotrain).unwrap();

        let cli_order_add_args = CliOrderAddArgs {
            no_broadcast: false,
            dotrain_file: dotrain_path.into(),
            deployment: "some-deployment".to_string(),
            skip_version_check: false,
            transaction_args: CliTransactionArgs {
                orderbook_address: Address::random(),
                derivation_index: None,
                chain_id: Some(123),
                rpc_url: "https://some-rpc.com".to_string(),
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                gas_fee_speed: None,
            },
        };
        let result = cli_order_add_args.to_add_order_args().await;

        // rm test file
        std::fs::remove_file(dotrain_path).unwrap();

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mismatch raindex version"));
    }

    fn get_dotrain(raindex_version: &str) -> String {
        format!(
            r#"
raindex-version: {raindex_version}
networks:
    some-network:
        rpc: https://some-rpc.com
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

metaboards:
    some-metaboard: https://www.some-metaboard.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
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

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        deployer: some-deployer
        orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            raindex_version = raindex_version
        )
        .to_string()
    }
}
