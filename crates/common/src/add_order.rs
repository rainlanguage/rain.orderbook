use crate::{
    dotrain_order::DotrainOrderError,
    rainlang::compose_to_rainlang,
    transaction::{TransactionArgs, TransactionArgsError},
};
use alloy::primitives::{hex::FromHexError, private::rand, Address, U256};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::{
    ReadContractParameters, ReadableClientError, ReadableClientHttp, WritableClientError,
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
                let client = ReadableClientHttp::new_from_url(input_token.network.rpc.to_string())?;
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
                let client =
                    ReadableClientHttp::new_from_url(output_token.network.rpc.to_string())?;
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
        rpc_url: String,
        rainlang: String,
    ) -> Result<Vec<u8>, AddOrderArgsError> {
        let client = ReadableClientHttp::new_from_url(rpc_url)
            .map_err(AddOrderArgsError::ReadableClientError)?;
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
    pub async fn try_into_call(&self, rpc_url: String) -> Result<addOrder2Call, AddOrderArgsError> {
        let rainlang = self.compose_to_rainlang()?;
        let bytecode = self
            .try_parse_rainlang(rpc_url.clone(), rainlang.clone())
            .await?;

        let meta = self.try_generate_meta(rainlang)?;

        let deployer = self.deployer;
        let dispair =
            DISPair::from_deployer(deployer, ReadableClientHttp::new_from_url(rpc_url.clone())?)
                .await?;

        // get the evaluable for the post action
        let post_rainlang = self.compose_addorder_post_task()?;
        let post_bytecode = self
            .try_parse_rainlang(rpc_url.clone(), post_rainlang.clone())
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

    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<addOrder2Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), AddOrderArgsError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let add_order_call = self.try_into_call(transaction_args.clone().rpc_url).await?;
        let params = transaction_args
            .try_into_write_contract_parameters(add_order_call, transaction_args.orderbook_address)
            .await?;

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
            .try_into_call(transaction_args.clone().rpc_url)
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
        let mut forker = Forker::new_with_fork(
            NewForkedEvm {
                fork_url: transaction_args.rpc_url.clone(),
                fork_block_number: None,
            },
            None,
            None,
        )
        .await?;
        let call = self.try_into_call(transaction_args.rpc_url.clone()).await?;
        forker
            .alloy_call_committing(
                Address::from(from_address),
                transaction_args.orderbook_address,
                call,
                U256::ZERO,
                true,
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dotrain_order::DotrainOrder;

    use super::*;
    use rain_orderbook_app_settings::{
        deployer::DeployerCfg,
        network::NetworkCfg,
        order::{OrderCfg, OrderIOCfg},
        scenario::ScenarioCfg,
        token::TokenCfg,
    };
    use rain_orderbook_test_fixtures::LocalEvm;
    use std::sync::{Arc, RwLock};
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
    fn test_order_config_v2_validity() {
        let inputs = vec![
            IO {
                token: Address::random(),
                vaultId: U256::from(0),
                decimals: 18,
            },
            IO {
                token: Address::random(),
                vaultId: U256::from(1),
                decimals: 18,
            },
        ];
        let outputs = vec![
            IO {
                token: Address::random(),
                vaultId: U256::from(0),
                decimals: 18,
            },
            IO {
                token: Address::random(),
                vaultId: U256::from(1),
                decimals: 18,
            },
        ];
        let interpreter = Address::random();
        let store = Address::random();

        let bytecode = vec![
            0x60, 0x60, 0x60, 0x40, 0x60, 0x60, 0x60, 0x40, 0x60, 0x60, 0x60, 0x40, 0x60, 0x60,
            0x60, 0x40,
        ];
        let meta = vec![9, 10, 11, 12];

        let order_config_v2 = OrderConfigV3 {
            nonce: U256::from(8).into(),
            secret: U256::from(8).into(),
            validInputs: inputs.clone(),
            validOutputs: outputs.clone(),
            evaluable: EvaluableV3 {
                interpreter,
                store,
                bytecode: bytecode.clone().into(),
            },
            meta: meta.clone().into(),
        };

        assert_eq!(order_config_v2.validInputs, inputs);
        assert_eq!(order_config_v2.validOutputs, outputs);
        assert_eq!(order_config_v2.evaluable.interpreter, interpreter);
        assert_eq!(order_config_v2.evaluable.store, store);
        assert_eq!(order_config_v2.evaluable.bytecode, bytecode);
        assert_eq!(order_config_v2.meta, meta.clone());
    }

    #[tokio::test]
    async fn test_add_order_random_vault_id_generation() {
        let network = NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-network".to_string(),
            rpc: Url::parse("https://some-rpc.com").unwrap(),
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
            r#"
raindex-version: {raindex_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
"#,
            raindex_version = "1234"
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
            rpc: Url::parse(&local_evm.url()).unwrap(),
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
            r#"
raindex-version: {raindex_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
"#,
            raindex_version = "1234"
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();

        let add_order_call = result.try_into_call(local_evm.url()).await.unwrap();

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
            rpc: Url::parse("https://some-rpc.com").unwrap(),
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
            r#"
raindex-version: {raindex_version}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 0 0;
"#,
            raindex_version = "1234"
        );
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment.clone())
            .await
            .unwrap();

        let post_action = result.compose_addorder_post_task().unwrap();

        assert_eq!(post_action, "/* 0. handle-add-order */ \n_ _: 0 0;");
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
networks:
    some-key:
        rpc: {rpc_url}
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
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order
            .dotrain_yaml()
            .get_deployment("some-key")
            .await
            .unwrap();
        AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .simulate_execute(
                TransactionArgs {
                    orderbook_address: *orderbook.address(),
                    rpc_url: local_evm.url(),
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
networks:
    some-key:
        rpc: {rpc_url}
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
        );

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order
            .dotrain_yaml()
            .get_deployment("some-key")
            .await
            .unwrap();
        AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .simulate_execute(
                TransactionArgs {
                    // send the tx to random address
                    orderbook_address: Address::random(),
                    rpc_url: local_evm.url(),
                    ..Default::default()
                },
                Some(token1_holder),
            )
            .await
            .expect_err("expected to fail but resolved");
    }
}
