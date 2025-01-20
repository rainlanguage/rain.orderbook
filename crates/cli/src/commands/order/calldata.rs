use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use alloy::sol_types::SolCall;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Clone)]
pub struct AddOrderCalldata {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for AddOrderCalldata {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let order = DotrainOrder::new(dotrain, settings.map(|v| vec![v])).await?;
        let dotrain_string = order.dotrain().to_string();

        let config_deployment = order.dotrain_yaml().get_deployment(&self.deployment)?;

        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain_string, config_deployment.clone()).await;

        let add_order_calldata = add_order_args?
            .try_into_call(config_deployment.scenario.deployer.network.rpc.to_string())
            .await?
            .abi_encode();

        output(&None, self.encoding.clone(), &add_order_calldata)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{hex::encode_prefixed, Address, Bytes, B256};
    use alloy::sol_types::SolValue;
    use alloy_ethers_typecast::rpc::Response;
    use clap::CommandFactory;
    use httpmock::MockServer;
    use std::str::FromStr;

    #[test]
    fn verify_cli() {
        AddOrderCalldata::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let dotrain_file = PathBuf::from_str("./some/dotrain_file.rain").unwrap();
        let settings_file = PathBuf::from_str("./some/settings_file.rain").unwrap();
        let deployment_str = "some-deployment";
        let output_str = "hex";

        let cmd = AddOrderCalldata::command();
        let result = cmd
            .try_get_matches_from(vec![
                "cmd",
                "-f",
                dotrain_file.to_str().unwrap(),
                "-c",
                settings_file.to_str().unwrap(),
                "-e",
                deployment_str,
                "-o",
                output_str,
            ])
            .unwrap();
        assert_eq!(
            result.get_one::<PathBuf>("dotrain_file"),
            Some(&dotrain_file)
        );
        assert_eq!(
            result.get_one::<PathBuf>("settings_file"),
            Some(&settings_file)
        );
        assert_eq!(
            result.get_one::<String>("deployment"),
            Some(&deployment_str.to_string())
        );
        assert_eq!(
            result.get_one::<SupportedOutputEncoding>("encoding"),
            Some(&SupportedOutputEncoding::Hex)
        );
    }

    #[tokio::test]
    async fn test_execute() {
        let rpc_server = MockServer::start_async().await;
        let dotrain = format!(
            "
raindex-version: {raindex_version}
networks:
    some-network:
        rpc: {}
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
            key: 10

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
#key !Test binding key
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;",
            rpc_server.url("/rpc").as_str(),
            raindex_version = "1234"
        );

        let dotrain_path = "./test_dotrain1.rain";
        std::fs::write(dotrain_path, dotrain).unwrap();

        // mock rpc response data
        // mock iInterpreter() call
        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0xf0cfdd37");
            then.body(
                Response::new_success(
                    1,
                    &B256::left_padding_from(Address::random().as_slice()).to_string(),
                )
                .to_json_string()
                .unwrap(),
            );
        });
        // mock iStore() call
        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0xc19423bc");
            then.body(
                Response::new_success(
                    2,
                    &B256::left_padding_from(Address::random().as_slice()).to_string(),
                )
                .to_json_string()
                .unwrap(),
            );
        });
        // mock iParser() call
        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0x24376855");
            then.body(
                Response::new_success(
                    3,
                    &B256::left_padding_from(Address::random().as_slice()).to_string(),
                )
                .to_json_string()
                .unwrap(),
            );
        });
        // mock parse2() call
        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0xa3869e14");
            then.body(
                Response::new_success(4, &encode_prefixed(Bytes::from(vec![1, 2]).abi_encode()))
                    .to_json_string()
                    .unwrap(),
            );
        });

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: dotrain_path.into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should succeed without err
        add_order_calldata.execute().await.unwrap();

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }
}
