use crate::{
    dotrain_order::DotrainOrderError,
    rainlang::compose_to_rainlang,
    transaction::{TransactionArgs, TransactionArgsError},
};
use alloy::primitives::{hex::FromHexError, private::rand, Address, U256};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::{
    ReadContractParameters, ReadableClientError, ReadableClientHttp, WritableClientError,
    WriteContractParameters,
};
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::transaction::{WriteTransaction, WriteTransactionStatus};
use dotrain::error::ComposeError;
use rain_interpreter_dispair::{DISPair, DISPairError};
#[cfg(not(target_family = "wasm"))]
use rain_interpreter_eval::{
    error::ForkCallError,
    fork::{Forker, NewForkedEvm},
};
use rain_interpreter_parser::{Parser2, ParserError, ParserV2};
use rain_metadata::{
    ContentEncoding, ContentLanguage, ContentType, Error as RainMetaError, KnownMagic,
    RainMetaDocumentV1Item,
};
use rain_orderbook_app_settings::deployment::DeploymentCfg;
use rain_orderbook_bindings::{
    IOrderBookV4::{addOrder2Call, EvaluableV3, OrderConfigV3, TaskV1, IO},
    ERC20::decimalsCall,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use thiserror::Error;

pub static ORDERBOOK_ORDER_ENTRYPOINTS: [&str; 2] = ["calculate-io", "handle-io"];
pub static ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS: [&str; 1] = ["handle-add-order"];

#[derive(Error, Debug)]
pub enum AddOrderArgsError {
    #[error("Empty Front Matter")]
    EmptyFrontmatter,
    #[error(transparent)]
    DISPairError(#[from] DISPairError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[error(transparent)]
    RainMetaError(#[from] RainMetaError),
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
    #[error("Input token not found for index: {0}")]
    InputTokenNotFound(String),
    #[error("Output token not found for index: {0}")]
    OutputTokenNotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename = "kebab-case")]
pub struct AddOrderArgs {
    pub dotrain: String,
    pub inputs: Vec<IO>,
    pub outputs: Vec<IO>,
    pub deployer: Address,
    pub bindings: HashMap<String, String>,
}

impl AddOrderArgs {
    /// create a new  instance from Deployment
    pub async fn new_from_deployment(
        dotrain: String,
        deployment: DeploymentCfg,
    ) -> Result<AddOrderArgs, AddOrderArgsError> {
        let random_vault_id: U256 = rand::random();

        let client = ReadableClientHttp::new_from_urls(
            deployment
                .order
                .network
                .rpcs
                .iter()
                .map(|rpc| rpc.to_string())
                .collect::<Vec<String>>(),
        )?;

        let mut inputs = vec![];
        for (i, input) in deployment.order.inputs.iter().enumerate() {
            let input_token = input
                .token
                .as_ref()
                .ok_or_else(|| AddOrderArgsError::InputTokenNotFound(i.to_string()))?;

            if let Some(decimals) = input_token.decimals {
                inputs.push(IO {
                    token: input_token.address,
                    vaultId: input.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            } else {
                let parameters = ReadContractParameters {
                    address: input_token.address,
                    call: decimalsCall {},
                    block_number: None,
                    gas: None,
                };
                let decimals = client.read(parameters).await?._0;
                inputs.push(IO {
                    token: input_token.address,
                    vaultId: input.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            }
        }

        let mut outputs = vec![];
        for (i, output) in deployment.order.outputs.iter().enumerate() {
            let output_token = output
                .token
                .as_ref()
                .ok_or_else(|| AddOrderArgsError::OutputTokenNotFound(i.to_string()))?;

            if let Some(decimals) = output_token.decimals {
                outputs.push(IO {
                    token: output_token.address,
                    vaultId: output.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            } else {
                let parameters = ReadContractParameters {
                    address: output_token.address,
                    call: decimalsCall {},
                    block_number: None,
                    gas: None,
                };
                let decimals = client.read(parameters).await?._0;
                outputs.push(IO {
                    token: output_token.address,
                    vaultId: output.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            }
        }

        Ok(AddOrderArgs {
            dotrain: dotrain.to_string(),
            inputs,
            outputs,
            deployer: deployment.scenario.deployer.address,
            bindings: deployment.scenario.bindings.to_owned(),
        })
    }

    /// Read parser address from deployer contract, then call parser to parse rainlang into bytecode and constants
    async fn try_parse_rainlang(
        &self,
        rpcs: Vec<String>,
        rainlang: String,
    ) -> Result<Vec<u8>, AddOrderArgsError> {
        let client = ReadableClientHttp::new_from_urls(rpcs)?;
        let dispair = DISPair::from_deployer(self.deployer, client.clone())
            .await
            .map_err(AddOrderArgsError::DISPairError)?;

        let parser: ParserV2 = dispair.clone().into();
        let rainlang_parsed = parser
            .parse_text(rainlang.as_str(), client)
            .await
            .map_err(AddOrderArgsError::ParserError)?;

        Ok(rainlang_parsed.bytecode.into())
    }

    /// Generate RainlangSource meta
    fn try_generate_meta(&self, rainlang: String) -> Result<Vec<u8>, AddOrderArgsError> {
        let meta_doc = RainMetaDocumentV1Item {
            payload: ByteBuf::from(rainlang.as_bytes()),
            magic: KnownMagic::RainlangSourceV1,
            content_type: ContentType::OctetStream,
            content_encoding: ContentEncoding::None,
            content_language: ContentLanguage::None,
        };
        let meta_doc_bytes = RainMetaDocumentV1Item::cbor_encode_seq(
            &vec![meta_doc],
            KnownMagic::RainMetaDocumentV1,
        )
        .map_err(AddOrderArgsError::RainMetaError)?;

        Ok(meta_doc_bytes)
    }

    /// Compose to rainlang string
    pub fn compose_to_rainlang(&self) -> Result<String, AddOrderArgsError> {
        let res = compose_to_rainlang(
            self.dotrain.clone(),
            self.bindings.clone(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )?;
        Ok(res)
    }

    /// Compose the addOrder2 post action
    pub fn compose_addorder_post_task(&self) -> Result<String, AddOrderArgsError> {
        let res = compose_to_rainlang(
            self.dotrain.clone(),
            self.bindings.clone(),
            &ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS,
        )?;
        Ok(res)
    }

    /// Generate an addOrder call from given dotrain
    pub async fn try_into_call(
        &self,
        rpcs: Vec<String>,
    ) -> Result<addOrder2Call, AddOrderArgsError> {
        let rainlang = self.compose_to_rainlang()?;
        let bytecode = self
            .try_parse_rainlang(rpcs.clone(), rainlang.clone())
            .await?;

        let meta = self.try_generate_meta(rainlang)?;

        let deployer = self.deployer;
        let dispair =
            DISPair::from_deployer(deployer, ReadableClientHttp::new_from_urls(rpcs.clone())?)
                .await?;

        // get the evaluable for the post action
        let post_rainlang = self.compose_addorder_post_task()?;
        let post_bytecode = self
            .try_parse_rainlang(rpcs.clone(), post_rainlang.clone())
            .await?;

        let post_evaluable = EvaluableV3 {
            interpreter: dispair.interpreter,
            store: dispair.store,
            bytecode: post_bytecode.into(),
        };

        let post_task = TaskV1 {
            evaluable: post_evaluable,
            signedContext: vec![],
        };

        Ok(addOrder2Call {
            config: OrderConfigV3 {
                validInputs: self.inputs.clone(),
                validOutputs: self.outputs.clone(),
                evaluable: EvaluableV3 {
                    interpreter: dispair.interpreter,
                    store: dispair.store,
                    bytecode: bytecode.into(),
                },
                meta: meta.into(),
                nonce: alloy::primitives::private::rand::random::<U256>().into(),
                secret: alloy::primitives::private::rand::random::<U256>().into(),
            },
            tasks: vec![post_task],
        })
    }

    pub async fn get_add_order_call_parameters(
        &self,
        transaction_args: TransactionArgs,
    ) -> Result<WriteContractParameters<addOrder2Call>, AddOrderArgsError> {
        let add_order_call = self.try_into_call(transaction_args.clone().rpcs).await?;
        let params = transaction_args.try_into_write_contract_parameters(
            add_order_call,
            transaction_args.orderbook_address,
        )?;
        Ok(params)
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<addOrder2Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), AddOrderArgsError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let params = self.get_add_order_call_parameters(transaction_args).await?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_add_order_calldata(
        &self,
        transaction_args: TransactionArgs,
    ) -> Result<Vec<u8>, AddOrderArgsError> {
        Ok(self
            .try_into_call(transaction_args.clone().rpcs)
            .await?
            .abi_encode())
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn simulate_execute(
        &self,
        transaction_args: TransactionArgs,
        from: Option<Address>,
    ) -> Result<(), AddOrderArgsError> {
        let from_address = if let Some(v) = from {
            v.0 .0
        } else {
            transaction_args
                .clone()
                .try_into_ledger_client()
                .await?
                .client
                .address()
                .0
        };

        let mut err: Option<AddOrderArgsError> = None;
        for rpc in transaction_args.rpcs.clone() {
            match Forker::new_with_fork(
                NewForkedEvm {
                    fork_url: rpc,
                    fork_block_number: None,
                },
                None,
                None,
            )
            .await
            {
                Ok(mut forker) => {
                    let call = self.try_into_call(transaction_args.clone().rpcs).await?;
                    forker
                        .alloy_call_committing(
                            Address::from(from_address),
                            transaction_args.orderbook_address,
                            call,
                            U256::ZERO,
                            true,
                        )
                        .await?;
                    break;
                }
                Err(e) => {
                    err = Some(AddOrderArgsError::ForkCallError(e));
                }
            }
        }
        if let Some(err) = err {
            return Err(err);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dotrain_order::DotrainOrder;
    use alloy::primitives::Bytes;
    use rain_orderbook_app_settings::{
        deployer::DeployerCfg,
        network::NetworkCfg,
        order::{OrderCfg, OrderIOCfg},
        scenario::ScenarioCfg,
        spec_version::SpecVersion,
        token::TokenCfg,
        yaml::default_document,
    };
    use rain_orderbook_test_fixtures::LocalEvm;
    use std::{
        str::FromStr,
        sync::{Arc, RwLock},
    };
    use strict_yaml_rust::StrictYaml;
    use url::Url;

    #[test]
    fn test_try_generate_meta() {
        let dotrain_body = String::from(
            "
#calculate-io
max-amount: 100e18,
price: 2e18;

#handle-io
max-amount: 100e18,
price: 2e18;
",
        );
        let args = AddOrderArgs {
            dotrain: "".into(),
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
        };

        let meta_bytes = args.try_generate_meta(dotrain_body).unwrap();
        assert_eq!(
            meta_bytes,
            vec![
                255, 10, 137, 198, 116, 238, 120, 116, 163, 0, 88, 93, 10, 35, 99, 97, 108, 99,
                117, 108, 97, 116, 101, 45, 105, 111, 10, 109, 97, 120, 45, 97, 109, 111, 117, 110,
                116, 58, 32, 49, 48, 48, 101, 49, 56, 44, 10, 112, 114, 105, 99, 101, 58, 32, 50,
                101, 49, 56, 59, 10, 10, 35, 104, 97, 110, 100, 108, 101, 45, 105, 111, 10, 109,
                97, 120, 45, 97, 109, 111, 117, 110, 116, 58, 32, 49, 48, 48, 101, 49, 56, 44, 10,
                112, 114, 105, 99, 101, 58, 32, 50, 101, 49, 56, 59, 10, 1, 27, 255, 19, 16, 158,
                65, 51, 111, 242, 2, 120, 24, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110,
                47, 111, 99, 116, 101, 116, 45, 115, 116, 114, 101, 97, 109
            ]
        );
    }

    #[test]
    fn test_try_generate_meta_empty_dotrain() {
        let args = AddOrderArgs {
            dotrain: "".into(),
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
        };
        let meta_bytes = args.try_generate_meta("".to_string()).unwrap();
        assert_eq!(
            meta_bytes,
            vec![
                255, 10, 137, 198, 116, 238, 120, 116, 163, 0, 64, 1, 27, 255, 19, 16, 158, 65, 51,
                111, 242, 2, 120, 24, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 111,
                99, 116, 101, 116, 45, 115, 116, 114, 101, 97, 109
            ]
        );
    }

    #[tokio::test]
    async fn test_add_order_random_vault_id_generation() {
        let network = NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-network".to_string(),
            rpcs: vec![Url::parse("https://some-rpc.com").unwrap()],
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: network_arc.clone(),
            address: Address::default(),
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token3".to_string()),
        };
        let token1_arc = Arc::new(token1);
        let token2_arc = Arc::new(token2);
        let token3_arc = Arc::new(token3);
        let known_vault_id = U256::from(1);
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            inputs: vec![
                OrderIOCfg {
                    token: Some(token1_arc.clone()),
                    vault_id: None,
                },
                OrderIOCfg {
                    token: Some(token2_arc.clone()),
                    vault_id: Some(known_vault_id),
                },
            ],
            outputs: vec![OrderIOCfg {
                token: Some(token3_arc.clone()),
                vault_id: None,
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = DeploymentCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();

        // input1 vault id should be same as known_vault_id
        assert_eq!(result.inputs[1].vaultId, known_vault_id);

        // input0 and output0 vaults should be the same random value
        assert_eq!(result.inputs[0].vaultId, result.outputs[0].vaultId);
    }

    #[tokio::test]
    async fn test_into_add_order_call() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let network = NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-network".to_string(),
            rpcs: vec![Url::parse(&local_evm.url()).unwrap()],
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: network_arc.clone(),
            address: *local_evm.deployer.address(),
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token3".to_string()),
        };
        let token1_arc = Arc::new(token1);
        let token2_arc = Arc::new(token2);
        let token3_arc = Arc::new(token3);
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            inputs: vec![
                OrderIOCfg {
                    token: Some(token1_arc.clone()),
                    vault_id: Some(U256::from(2)),
                },
                OrderIOCfg {
                    token: Some(token2_arc.clone()),
                    vault_id: Some(U256::from(1)),
                },
            ],
            outputs: vec![OrderIOCfg {
                token: Some(token3_arc.clone()),
                vault_id: Some(U256::from(4)),
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = DeploymentCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );

        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();

        let add_order_call = result.try_into_call(vec![local_evm.url()]).await.unwrap();

        assert_eq!(add_order_call.config.validInputs.len(), 2);
        assert_eq!(add_order_call.config.validOutputs.len(), 1);
        assert_eq!(add_order_call.tasks.len(), 1);

        assert_eq!(add_order_call.config.validInputs[0].vaultId, U256::from(2));
        assert_eq!(add_order_call.config.validInputs[1].vaultId, U256::from(1));
        assert_eq!(add_order_call.config.validOutputs[0].vaultId, U256::from(4));

        assert_eq!(add_order_call.tasks[0].evaluable.bytecode.len(), 111);

        assert_eq!(add_order_call.config.meta.len(), 105);

        assert_eq!(
            add_order_call.config.validInputs[0].token,
            Address::default()
        );

        assert_eq!(
            add_order_call.config.validInputs[1].token,
            Address::default()
        );

        assert_eq!(
            add_order_call.config.validOutputs[0].token,
            Address::default()
        );

        assert_eq!(
            add_order_call.tasks[0].evaluable.interpreter,
            *local_evm.interpreter.address()
        );

        assert_eq!(
            add_order_call.tasks[0].evaluable.store,
            *local_evm.store.address()
        );

        assert_eq!(add_order_call.tasks[0].evaluable.bytecode.len(), 111);
        assert_eq!(add_order_call.tasks[0].signedContext.len(), 0);
    }

    #[tokio::test]
    async fn test_add_order_post_action() {
        let network = NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-network".to_string(),
            rpcs: vec![Url::parse("https://some-rpc.com").unwrap()],
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: network_arc.clone(),
            address: Address::default(),
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token3".to_string()),
        };
        let token1_arc = Arc::new(token1);
        let token2_arc = Arc::new(token2);
        let token3_arc = Arc::new(token3);
        let known_vault_id = U256::from(1);
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            inputs: vec![
                OrderIOCfg {
                    token: Some(token1_arc.clone()),
                    vault_id: None,
                },
                OrderIOCfg {
                    token: Some(token2_arc.clone()),
                    vault_id: Some(known_vault_id),
                },
            ],
            outputs: vec![OrderIOCfg {
                token: Some(token3_arc.clone()),
                vault_id: None,
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = DeploymentCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment.clone())
            .await
            .unwrap();

        let post_action = result.compose_addorder_post_task().unwrap();

        assert_eq!(post_action, "/* 0. handle-add-order */ \n_ _: 0 0;");
    }

    #[tokio::test]
    async fn test_compose_addorder_post_task_empty_dotrain() {
        let local_evm = LocalEvm::new().await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let result = AddOrderArgs::new_from_deployment("".to_string(), deployment.clone())
            .await
            .unwrap();
        let err = result.compose_addorder_post_task().unwrap_err();
        assert!(matches!(
            err,
            AddOrderArgsError::ComposeError(ComposeError::Reject(_))
        ));
    }

    #[tokio::test]
    async fn test_compose_addorder_post_task_missing_bindings() {
        let local_evm = LocalEvm::new().await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let result = AddOrderArgs::new_from_deployment(
            format!(
                "
version: {spec_version}
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-add-order
_ _: 0 key1;
",
                spec_version = SpecVersion::current()
            )
            .to_string(),
            deployment.clone(),
        )
        .await
        .unwrap();
        let err = result.compose_addorder_post_task().unwrap_err();
        assert!(matches!(
            err,
            AddOrderArgsError::ComposeError(ComposeError::Problems(_))
        ));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_simulate_execute_ok() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
deployers:
    some-key:
        address: {deployer}
tokens:
    t1:
        network: some-key
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
    t2:
        network: some-key
        address: {token1}
        decimals: 18
        label: Token1
        symbol: token1
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: t1
        outputs:
            - token: t2
              vault-id: 0x01
scenarios:
    some-key:
        deployer: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
_ _: 16 52;
#handle-add-order
:;
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            deployer = local_evm.deployer.address(),
            token1 = token1.address(),
            token2 = token2.address(),
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .simulate_execute(
                TransactionArgs {
                    orderbook_address: *orderbook.address(),
                    rpcs: vec![local_evm.url()],
                    ..Default::default()
                },
                Some(token1_holder),
            )
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_simulate_execute_err() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
deployers:
    some-key:
        address: {deployer}
tokens:
    t1:
        network: some-key
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
    t2:
        network: some-key
        address: {token1}
        decimals: 18
        label: Token1
        symbol: token1
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: t1
        outputs:
            - token: t2
              vault-id: 0x01
scenarios:
    some-key:
        deployer: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
_ _: 16 52;
#handle-add-order
:;
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            deployer = local_evm.deployer.address(),
            token1 = token1.address(),
            token2 = token2.address(),
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .simulate_execute(
                TransactionArgs {
                    // send the tx to random address
                    orderbook_address: Address::random(),
                    rpcs: vec![local_evm.url()],
                    ..Default::default()
                },
                Some(token1_holder),
            )
            .await
            .expect_err("expected to fail but resolved");
    }

    fn get_deployment(rpc_url: &str, deployer: Address) -> DeploymentCfg {
        let network = NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-network".to_string(),
            rpcs: vec![Url::parse(rpc_url).unwrap()],
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = DeployerCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: network_arc.clone(),
            address: deployer,
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token3".to_string()),
        };
        let token1_arc = Arc::new(token1);
        let token2_arc = Arc::new(token2);
        let token3_arc = Arc::new(token3);
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            inputs: vec![
                OrderIOCfg {
                    token: Some(token1_arc.clone()),
                    vault_id: Some(U256::from(2)),
                },
                OrderIOCfg {
                    token: Some(token2_arc.clone()),
                    vault_id: Some(U256::from(1)),
                },
            ],
            outputs: vec![OrderIOCfg {
                token: Some(token3_arc.clone()),
                vault_id: Some(U256::from(4)),
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        DeploymentCfg {
            document: default_document(),
            key: "".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        }
    }

    #[tokio::test]
    async fn test_try_parse_rainlang() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());

        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let add_order_args = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let rainlang = add_order_args.compose_to_rainlang().unwrap();
        let res = add_order_args
            .try_parse_rainlang(vec![local_evm.url()], rainlang)
            .await
            .unwrap();
        assert_eq!(
            res,
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 2, 0, 0, 0, 12, 2, 2, 0, 2, 1, 16, 0, 0, 1,
                16, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[tokio::test]
    async fn test_try_parse_rainlang_invalid_url() {
        let deployment = get_deployment("https://testtest.com", Address::random());
        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let add_order_args = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let rainlang = add_order_args.compose_to_rainlang().unwrap();
        let err = add_order_args
            .try_parse_rainlang(vec!["invalid-url".to_string()], rainlang)
            .await
            .unwrap_err();
        assert!(matches!(err, AddOrderArgsError::ReadableClientError(_)));
    }

    #[tokio::test]
    async fn test_try_parse_rainlang_missing_rpc_data() {
        let rpc_url = "https://testtest.com/".to_string();
        let deployment = get_deployment(&rpc_url, Address::random());
        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let add_order_args = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let rainlang = add_order_args.compose_to_rainlang().unwrap();
        let err = add_order_args
            .try_parse_rainlang(vec![rpc_url.clone()], rainlang)
            .await
            .unwrap_err();
        assert!(
            matches!(
                &err,
                AddOrderArgsError::DISPairError(DISPairError::ReadableClientError(
                    ReadableClientError::AllProvidersFailed(ref msg)
                ))
                if msg.get(&rpc_url).is_some()
                    && matches!(
                        msg.get(&rpc_url).unwrap(),
                        ReadableClientError::ReadCallError(_)
                    )
            ),
            "unexpected error variant: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_try_parse_rainlang_malformed_rainlang() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let add_order_args = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let rainlang = add_order_args.compose_to_rainlang().unwrap();
        let err = add_order_args
            .try_parse_rainlang(vec![local_evm.url()], rainlang.as_str()[..10].to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, AddOrderArgsError::ParserError(_)));
    }

    #[tokio::test]
    async fn test_compose_to_rainlang() {
        let local_evm = LocalEvm::new().await;
        let dotrain = format!(
            r#"
networks:
    test:
        rpcs:
            - {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    test:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    test:
        bindings:
            key1: 10
            key2: 20
        deployer: test
---
#key1 !Test binding
#key2 !Test binding
#calculate-io
_ _: key1 key2;
#handle-io
:;
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
        );

        let add_order_args = AddOrderArgs {
            dotrain: dotrain.clone(),
            inputs: vec![],
            outputs: vec![],
            deployer: *local_evm.deployer.address(),
            bindings: HashMap::from([
                ("key1".to_string(), "10".to_string()),
                ("key2".to_string(), "20".to_string()),
            ]),
        };
        let rainlang = add_order_args.compose_to_rainlang().unwrap();
        assert_eq!(
            rainlang,
            "/* 0. calculate-io */ \n_ _: 10 20;\n\n/* 1. handle-io */ \n:;"
        );
    }

    #[tokio::test]
    async fn test_compose_to_rainlang_invalid_dotrain() {
        let add_order_args = AddOrderArgs {
            dotrain: "invalid-dotrain".to_string(),
            inputs: vec![],
            outputs: vec![],
            deployer: Address::random(),
            bindings: HashMap::from([
                ("key1".to_string(), "10".to_string()),
                ("key2".to_string(), "20".to_string()),
            ]),
        };
        let err = add_order_args.compose_to_rainlang().unwrap_err();
        assert!(matches!(
            err,
            AddOrderArgsError::ComposeError(ComposeError::Problems(_))
        ));
    }

    #[tokio::test]
    async fn test_compose_to_rainlang_missing_bindings() {
        let dotrain = r#"
networks:
    test:
        rpcs:
            - https://testtest.com
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    test:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    test:
        bindings:
            key1: 10
            key2: 20
        deployer: test
---
#key1 !Test binding
#key2 !Test binding
#calculate-io
_ _: key1 key2;
#handle-io
:;
#handle-add-order
:;"#;
        let add_order_args = AddOrderArgs {
            dotrain: dotrain.to_string(),
            inputs: vec![],
            outputs: vec![],
            deployer: Address::random(),
            bindings: HashMap::new(),
        };
        let err = add_order_args.compose_to_rainlang().unwrap_err();
        assert!(matches!(
            err,
            AddOrderArgsError::ComposeError(ComposeError::Problems(_))
        ));
    }

    #[tokio::test]
    async fn test_get_add_order_call_parameters() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let dotrain = format!(
            r#"
networks:
    test:
        rpcs:
            - {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    test:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    test:
        deployer: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
        );
        let add_order_args = AddOrderArgs {
            dotrain: dotrain.to_string(),
            inputs: vec![IO {
                token: *local_evm.tokens[0].address(),
                decimals: 18,
                vaultId: U256::from(2),
            }],
            outputs: vec![IO {
                token: *local_evm.tokens[1].address(),
                decimals: 18,
                vaultId: U256::from(4),
            }],
            deployer: *local_evm.deployer.address(),
            bindings: HashMap::new(),
        };

        let add_order_call = addOrder2Call {
            config: OrderConfigV3 {
                evaluable: EvaluableV3 {
                    interpreter: *local_evm.interpreter.address(),
                    store: *local_evm.store.address(),
                    bytecode: Bytes::from_str(
                        "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000",
                    )
                    .unwrap(),
                },
                validInputs: vec![IO {
                    token: *local_evm.tokens[0].address(),
                    decimals: 18,
                    vaultId: U256::from(2),
                }],
                validOutputs: vec![IO {
                    token: *local_evm.tokens[1].address(),
                    decimals: 18,
                    vaultId: U256::from(4),
                }],
                nonce: alloy::primitives::private::rand::random::<U256>().into(),
                secret: alloy::primitives::private::rand::random::<U256>().into(),
                meta: Bytes::from_str("0xff0a89c674ee7874a30058382f2a20302e2063616c63756c6174652d696f202a2f200a5f205f3a203020303b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d").unwrap(),
            },
            tasks: vec![
                TaskV1 {
                    evaluable: EvaluableV3 {
                        interpreter: *local_evm.interpreter.address(),
                        store: *local_evm.store.address(),
                        bytecode: Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000701000000000000").unwrap(),
                    },
                    signedContext: vec![],
                },
            ],
        };

        let res = add_order_args
            .get_add_order_call_parameters(TransactionArgs {
                rpcs: vec![local_evm.url().to_string()],
                orderbook_address: *local_evm.orderbook.address(),
                max_priority_fee_per_gas: Some(U256::from(100)),
                max_fee_per_gas: Some(U256::from(200)),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(res.call.config.evaluable, add_order_call.config.evaluable);
        assert_eq!(
            res.call.config.validInputs,
            add_order_call.config.validInputs
        );
        assert_eq!(
            res.call.config.validOutputs,
            add_order_call.config.validOutputs
        );
        assert_eq!(res.call.config.meta, add_order_call.config.meta);
        assert_eq!(res.call.tasks, add_order_call.tasks);
        assert_eq!(res.address, *local_evm.orderbook.address());
        assert_eq!(res.max_priority_fee_per_gas, Some(U256::from(100)));
        assert_eq!(res.max_fee_per_gas, Some(U256::from(200)));
    }

    #[tokio::test]
    async fn test_get_add_order_calldata() {
        let local_evm = LocalEvm::new().await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let calldata: Bytes = result
            .get_add_order_calldata(TransactionArgs {
                rpcs: vec![local_evm.url().to_string()],
                ..Default::default()
            })
            .await
            .unwrap()
            .into();

        let expected_bytes: Bytes = addOrder2Call {
            config: OrderConfigV3 {
                evaluable: EvaluableV3 {
                    interpreter: *local_evm.interpreter.address(),
                    store: *local_evm.store.address(),
                    bytecode: Bytes::from_str("0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000").unwrap(),
                },
                validInputs: vec![
                    IO {
                        token: Address::default(),
                        decimals: 18,
                        vaultId: U256::from(2),
                    },
                    IO {
                        token: Address::default(),
                        decimals: 18,
                        vaultId: U256::from(1),
                    },
                ],
                validOutputs: vec![IO {
                    token: Address::default(),
                    decimals: 18,
                    vaultId: U256::from(4),
                }],
                nonce: U256::from(0).into(),
                secret: U256::from(0).into(),
                meta: Bytes::from_str("0xff0a89c674ee7874a30058382f2a20302e2063616c63756c6174652d696f202a2f200a5f205f3a203020303b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d").unwrap(),
            },
            tasks: vec![TaskV1 {
                evaluable: EvaluableV3 {
                    interpreter: *local_evm.interpreter.address(),
                    store: *local_evm.store.address(),
                    bytecode: Bytes::from_str("0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f010000020200020110000001100000").unwrap(),
                },
                signedContext: vec![],
            }],
        }
        .abi_encode()
        .into();

        // Nonce and secret are random, so we can't compare the whole calldata
        assert_eq!(calldata[..164], expected_bytes[..164]);
        assert_eq!(calldata[228..], expected_bytes[228..]);
    }

    #[tokio::test]
    async fn test_get_add_order_calldata_invalid_rpc_url() {
        let local_evm = LocalEvm::new().await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let dotrain = format!(
            "
version: {spec_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
",
            spec_version = SpecVersion::current()
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();
        let rpc_url = "https://testtest.com/".to_string();
        let err = result
            .get_add_order_calldata(TransactionArgs {
                rpcs: vec![rpc_url.clone()],
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(
            matches!(
                &err,
                AddOrderArgsError::DISPairError(DISPairError::ReadableClientError(
                    ReadableClientError::AllProvidersFailed(msg)
                ))
                if msg.get(&rpc_url).is_some()
                    && matches!(
                        msg.get(&rpc_url).unwrap(),
                        ReadableClientError::ReadCallError(_)
                    )
            ),
            "unexpected error variant: {err:?}"
        );
    }
}
