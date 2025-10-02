use crate::{
    dotrain_order::DotrainOrderError,
    rainlang::compose_to_rainlang,
    transaction::{TransactionArgs, TransactionArgsError},
};
use alloy::primitives::FixedBytes;
#[cfg(not(target_family = "wasm"))]
use alloy::primitives::U256;
use alloy::primitives::{hex::FromHexError, Address, Bytes, B256};
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::{
    ReadableClient, ReadableClientError, WritableClientError, WriteContractParameters,
};
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::{WriteTransaction, WriteTransactionStatus};
use dotrain::error::ComposeError;
use rain_interpreter_bindings::IParserV2::parse2Return;
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
use rain_metadata_bindings::MetaBoard::emitMetaCall;
use rain_orderbook_app_settings::deployment::DeploymentCfg;
use rain_orderbook_bindings::IOrderBookV5::{
    addOrder3Call, EvaluableV4, OrderConfigV4, TaskV2, IOV2,
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
    DotrainOrderError(Box<DotrainOrderError>),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    ForkCallError(Box<ForkCallError>),
    #[error("Input token not found for index: {0}")]
    InputTokenNotFound(String),
    #[error("Output token not found for index: {0}")]
    OutputTokenNotFound(String),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}

impl From<DotrainOrderError> for AddOrderArgsError {
    fn from(err: DotrainOrderError) -> Self {
        Self::DotrainOrderError(Box::new(err))
    }
}

#[cfg(not(target_family = "wasm"))]
impl From<ForkCallError> for AddOrderArgsError {
    fn from(err: ForkCallError) -> Self {
        Self::ForkCallError(Box::new(err))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename = "kebab-case")]
pub struct AddOrderArgs {
    pub dotrain: String,
    pub inputs: Vec<IOV2>,
    pub outputs: Vec<IOV2>,
    pub deployer: Address,
    pub bindings: HashMap<String, String>,
    pub meta: Option<Vec<RainMetaDocumentV1Item>>,
}

impl AddOrderArgs {
    /// create a new  instance from Deployment
    pub async fn new_from_deployment(
        dotrain: String,
        deployment: DeploymentCfg,
        meta: Option<Vec<RainMetaDocumentV1Item>>,
    ) -> Result<AddOrderArgs, AddOrderArgsError> {
        let random_vault_id = B256::random();

        let mut inputs = vec![];
        for (i, input) in deployment.order.inputs.iter().enumerate() {
            let input_token = input
                .token
                .as_ref()
                .ok_or_else(|| AddOrderArgsError::InputTokenNotFound(i.to_string()))?;

            inputs.push(IOV2 {
                token: input_token.address,
                vaultId: input.vault_id.map(B256::from).unwrap_or(random_vault_id),
            });
        }

        let mut outputs = vec![];
        for (i, output) in deployment.order.outputs.iter().enumerate() {
            let output_token = output
                .token
                .as_ref()
                .ok_or_else(|| AddOrderArgsError::OutputTokenNotFound(i.to_string()))?;

            outputs.push(IOV2 {
                token: output_token.address,
                vaultId: output.vault_id.map(B256::from).unwrap_or(random_vault_id),
            });
        }

        Ok(AddOrderArgs {
            dotrain,
            inputs,
            outputs,
            deployer: deployment.scenario.deployer.address,
            bindings: deployment.scenario.bindings.to_owned(),
            meta,
        })
    }

    /// Read parser address from deployer contract, then call parser to parse rainlang into bytecode and constants
    async fn try_parse_rainlang(
        &self,
        rpcs: Vec<String>,
        rainlang: String,
    ) -> Result<Vec<u8>, AddOrderArgsError> {
        let client = ReadableClient::new_from_http_urls(rpcs.clone())?;
        let dispair = DISPair::from_deployer(self.deployer, client)
            .await
            .map_err(AddOrderArgsError::DISPairError)?;

        let client = ReadableClient::new_from_http_urls(rpcs)?;
        let parser: ParserV2 = dispair.clone().into();
        let rainlang_parsed: parse2Return = parser
            .parse_text(rainlang.as_str(), client)
            .await
            .map_err(AddOrderArgsError::ParserError)?;

        Ok(rainlang_parsed.bytecode.into())
    }

    /// Generate RainlangSource meta
    fn try_generate_meta(&self, rainlang: String) -> Result<Vec<u8>, AddOrderArgsError> {
        let mut meta_docs = Vec::new();

        let rainlang_meta_doc = RainMetaDocumentV1Item {
            payload: ByteBuf::from(rainlang.as_bytes()),
            magic: KnownMagic::RainlangSourceV1,
            content_type: ContentType::OctetStream,
            content_encoding: ContentEncoding::None,
            content_language: ContentLanguage::None,
        };
        meta_docs.push(rainlang_meta_doc);

        if let Some(existing_meta) = &self.meta {
            if !existing_meta.is_empty() {
                meta_docs.extend(existing_meta.iter().filter_map(|i| match i.magic {
                    KnownMagic::RainlangSourceV1 | KnownMagic::DotrainSourceV1 => None,
                    _ => Some(i.clone()),
                }));
            }
        }

        let meta_doc_bytes =
            RainMetaDocumentV1Item::cbor_encode_seq(&meta_docs, KnownMagic::RainMetaDocumentV1)
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

    pub async fn try_into_call(
        &self,
        rpcs: Vec<String>,
    ) -> Result<addOrder3Call, AddOrderArgsError> {
        let rainlang = self.compose_to_rainlang()?;
        let bytecode = self
            .try_parse_rainlang(rpcs.clone(), rainlang.clone())
            .await?;

        let meta = self.try_generate_meta(rainlang)?;

        let deployer = self.deployer;
        let dispair =
            DISPair::from_deployer(deployer, ReadableClient::new_from_http_urls(rpcs.clone())?)
                .await?;

        // get the evaluable for the post action
        let post_rainlang = self.compose_addorder_post_task()?;
        let post_bytecode = self
            .try_parse_rainlang(rpcs.clone(), post_rainlang.clone())
            .await?;

        let post_evaluable = EvaluableV4 {
            interpreter: dispair.interpreter,
            store: dispair.store,
            bytecode: post_bytecode.into(),
        };

        let post_task = TaskV2 {
            evaluable: post_evaluable,
            signedContext: vec![],
        };

        Ok(addOrder3Call {
            config: OrderConfigV4 {
                validInputs: self.inputs.clone(),
                validOutputs: self.outputs.clone(),
                evaluable: EvaluableV4 {
                    interpreter: dispair.interpreter,
                    store: dispair.store,
                    bytecode: bytecode.into(),
                },
                meta: meta.clone().into(),
                nonce: B256::random(),
                secret: B256::random(),
            },
            tasks: vec![post_task],
        })
    }

    pub fn try_into_emit_meta_call(&self) -> Result<Option<emitMetaCall>, AddOrderArgsError> {
        match self.meta.as_ref() {
            Some(meta_docs) => {
                match meta_docs
                    .iter()
                    .find(|document| document.magic == KnownMagic::DotrainGuiStateV1)
                {
                    Some(doc) => {
                        let subject_hash = doc.clone().hash(false)?;
                        let meta = RainMetaDocumentV1Item::cbor_encode_seq(
                            &vec![RainMetaDocumentV1Item {
                                payload: ByteBuf::from(self.dotrain.as_bytes()),
                                magic: KnownMagic::DotrainSourceV1,
                                content_type: ContentType::OctetStream,
                                content_encoding: ContentEncoding::None,
                                content_language: ContentLanguage::None,
                            }],
                            KnownMagic::RainMetaDocumentV1,
                        )?;
                        Ok(Some(emitMetaCall {
                            subject: FixedBytes::from(subject_hash),
                            meta: Bytes::copy_from_slice(&meta),
                        }))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    pub async fn get_add_order_call_parameters(
        &self,
        transaction_args: TransactionArgs,
    ) -> Result<WriteContractParameters<addOrder3Call>, AddOrderArgsError> {
        let add_order_call = self.try_into_call(transaction_args.clone().rpcs).await?;
        let params = transaction_args.try_into_write_contract_parameters(
            add_order_call,
            transaction_args.orderbook_address,
        )?;
        Ok(params)
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<addOrder3Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), AddOrderArgsError> {
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let params = self.get_add_order_call_parameters(transaction_args).await?;

        WriteTransaction::new(ledger_client, params, 4, transaction_status_changed)
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
            let (_, Address(FixedBytes(address))) =
                transaction_args.clone().try_into_ledger_client().await?;
            address
        };

        let mut err: Option<AddOrderArgsError> = None;

        if transaction_args.rpcs.is_empty() {
            return Err(AddOrderArgsError::InvalidArgs(
                "rpcs cannot be empty".into(),
            ));
        }
        for rpc in transaction_args.rpcs.clone() {
            match Forker::new_with_fork(
                NewForkedEvm {
                    fork_url: rpc.clone(),
                    fork_block_number: None,
                },
                None,
                None,
            )
            .await
            {
                Ok(mut forker) => {
                    let call = match self.try_into_call(vec![rpc.clone()]).await {
                        Ok(c) => c,
                        Err(e) => {
                            err = Some(e);
                            continue;
                        }
                    };
                    match forker
                        .alloy_call_committing(
                            Address::from(from_address),
                            transaction_args.orderbook_address,
                            call,
                            U256::ZERO,
                            true,
                        )
                        .await
                    {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            err = Some(AddOrderArgsError::ForkCallError(Box::new(e)));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    err = Some(AddOrderArgsError::ForkCallError(Box::new(e)));
                }
            }
        }
        if let Some(err) = err {
            return Err(err);
        }

        Ok(())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::dotrain_order::DotrainOrder;
    use alloy::{hex::FromHex, primitives::Bytes};
    use rain_metadata::types::dotrain::gui_state_v1::DotrainGuiStateV1;
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
        collections::BTreeMap,
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
            meta: None,
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
            meta: None,
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
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
            .await
            .unwrap();

        // input1 vault id should be same as known_vault_id
        assert_eq!(
            result.inputs[1].vaultId,
            B256::from(U256::from(known_vault_id))
        );

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

        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
            .await
            .unwrap();

        let add_order_call = result.try_into_call(vec![local_evm.url()]).await.unwrap();

        assert_eq!(add_order_call.config.validInputs.len(), 2);
        assert_eq!(add_order_call.config.validOutputs.len(), 1);
        assert_eq!(add_order_call.tasks.len(), 1);

        assert_eq!(
            add_order_call.config.validInputs[0].vaultId,
            B256::from(U256::from(2))
        );
        assert_eq!(
            add_order_call.config.validInputs[1].vaultId,
            B256::from(U256::from(1))
        );
        assert_eq!(
            add_order_call.config.validOutputs[0].vaultId,
            B256::from(U256::from(4))
        );

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

    #[test]
    fn test_try_into_emit_meta_call_with_gui_state() {
        let dotrain = "test dotrain".to_string();
        let meta_hex = "0xff0a89c674ee7874a3005908342f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d20307863436633453665313645343042323636313864323136623433383564303836363634383736373832203078653830653734333863653662313035356338653943444531623633333661344639443533433636360a616d6f756e742d65706f6368730a74726164652d65706f6368733a63616c6c3c323e28292c0a6d61782d6f75747075743a2063616c6c3c333e28616d6f756e742d65706f6368732074726164652d65706f636873292c0a696f3a2063616c6c3c343e2874726164652d65706f636873292c0a3a63616c6c3c353e28696f293b0a0a2f2a20312e2068616e646c652d696f202a2f200a6d696e2d616d6f756e743a206d756c283120302e39292c0a3a656e7375726528677265617465722d7468616e2d6f722d657175616c2d746f286f75747075742d7661756c742d64656372656173652829206d696e2d616d6f756e742920224d696e20747261646520616d6f756e742e22292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a3a7365742868617368286f726465722d6861736828292022616d6f756e742d757365642229206164642875736564206f75747075742d7661756c742d6465637265617365282929293b0a0a2f2a20322e206765742d65706f6368202a2f200a696e697469616c2d74696d653a2063616c6c3c363e28292c0a6c6173742d74696d65205f3a2063616c6c3c373e28292c0a6475726174696f6e3a20737562286e6f77282920616e79286c6173742d74696d6520696e697469616c2d74696d6529292c0a746f74616c2d6475726174696f6e3a20737562286e6f77282920696e697469616c2d74696d65292c0a726174696f2d667265657a652d616d6f756e742d65706f6368733a2064697628312031292c0a726174696f2d667265657a652d74726164652d65706f6368733a206d756c28726174696f2d667265657a652d616d6f756e742d65706f63687320646976283630203336303029292c0a616d6f756e742d65706f6368733a2064697628746f74616c2d6475726174696f6e203630292c0a74726164652d65706f6368733a206d617828302073756228646976286475726174696f6e20333630302920726174696f2d667265657a652d74726164652d65706f63687329293b0a0a2f2a20332e20616d6f756e742d666f722d65706f6368202a2f200a616d6f756e742d65706f6368730a74726164652d65706f6368733a2c0a746f74616c2d617661696c61626c653a206c696e6561722d67726f7774682830203120616d6f756e742d65706f636873292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a756e757365643a2073756228746f74616c2d617661696c61626c652075736564292c0a64656361793a2063616c6c3c383e2874726164652d65706f636873292c0a7368792d64656361793a20657665727928677265617465722d7468616e2874726164652d65706f63687320302e303529206465636179292c0a7661726961626c652d636f6d706f6e656e743a2073756228312031292c0a7461726765742d616d6f756e743a206164642831206d756c287661726961626c652d636f6d706f6e656e74207368792d646563617929292c0a6361707065642d756e757365643a206d696e28756e75736564207461726765742d616d6f756e74293b0a0a2f2a20342e20696f2d666f722d65706f6368202a2f200a65706f63683a2c0a6c6173742d696f3a2063616c6c3c373e28292c0a6d61782d6e6578742d74726164653a20616e79286d756c286c6173742d696f20312e3031292063616c6c3c393e2829292c0a626173656c696e652d6e6578742d74726164653a206d756c286c6173742d696f2030292c0a7265616c2d626173656c696e653a206d617828626173656c696e652d6e6578742d74726164652063616c6c3c31303e2829292c0a7661726961626c652d636f6d706f6e656e743a206d6178283020737562286d61782d6e6578742d7472616465207265616c2d626173656c696e6529292c0a61626f76652d626173656c696e653a206d756c287661726961626c652d636f6d706f6e656e742063616c6c3c383e2865706f636829292c0a5f3a20616464287265616c2d626173656c696e652061626f76652d626173656c696e65293b0a0a2f2a20352e207365742d6c6173742d7472616465202a2f200a6c6173742d696f3a2c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229206e6f772829292c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229206c6173742d696f293b0a0a2f2a20362e206765742d696e697469616c2d74696d65202a2f200a5f3a6765742868617368286f726465722d6861736828292022696e697469616c2d74696d652229293b0a0a2f2a20372e206765742d6c6173742d7472616465202a2f200a6c6173742d74696d653a6765742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229292c0a6c6173742d696f3a6765742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229293b0a0a2f2a20382e2068616c666c696665202a2f200a65706f63683a2c0a76616c3a20706f77657228302e352065706f6368293b0a0a2f2a20392e20636f6e7374616e742d696e697469616c2d696f202a2f200a5f3a20313b0a0a2f2a2031302e20636f6e7374616e742d626173656c696e65202a2f200a5f3a20313b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616da3005902b7a66c646f747261696e5f686173685820e50585fb1ace3108592f8b572e8e2c381a7c82c629e52b56108287505a07ae8f6c6669656c645f76616c756573a970616d6f756e742d7065722d65706f6368a362696470616d6f756e742d7065722d65706f6368646e616d65f66576616c7565613168626173656c696e65a362696468626173656c696e65646e616d65f66576616c756561316a696e697469616c2d696fa36269646a696e697469616c2d696f646e616d65f66576616c75656131706d61782d74726164652d616d6f756e74a3626964706d61782d74726164652d616d6f756e74646e616d65f66576616c75656131706d696e2d74726164652d616d6f756e74a3626964706d696e2d74726164652d616d6f756e74646e616d65f66576616c75656131781e6e6578742d74726164652d626173656c696e652d6d756c7469706c696572a36269646130646e616d65f66576616c75656130756e6578742d74726164652d6d756c7469706c696572a36269646130646e616d65f66576616c756564312e30317574696d652d7065722d616d6f756e742d65706f6368a36269646130646e616d65f66576616c75656236307474696d652d7065722d74726164652d65706f6368a36269646132646e616d65f66576616c75656433363030686465706f73697473a06d73656c6563745f746f6b656e73a265696e707574a2676e6574776f726b68617262697472756d67616464726573735415059c599c16fd8f70b633ade165502d6402cd49666f7574707574a2676e6574776f726b68617262697472756d67616464726573735409d4214c03d01f49544c0448dbe3a27f768f2b34697661756c745f696473a267696e7075745f306d30786239363066333038386330686f75747075745f306d307862393630663330383863307373656c65637465645f6465706c6f796d656e7468617262697472756d011bffda7b2fb167c2860278186170706c69636174696f6e2f6f637465742d73747265616d";
        let meta_bytes = Vec::<u8>::from_hex(meta_hex.trim_start_matches("0x")).unwrap();
        let meta_docs = RainMetaDocumentV1Item::cbor_decode(&meta_bytes).unwrap();
        let gui_state_doc = meta_docs
            .iter()
            .find(|doc| doc.magic == KnownMagic::DotrainGuiStateV1)
            .expect("expected GUI state doc")
            .clone();

        let args = AddOrderArgs {
            dotrain: dotrain.clone(),
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
            meta: Some(vec![gui_state_doc.clone()]),
        };

        let emit_meta_call = args
            .try_into_emit_meta_call()
            .unwrap()
            .expect("should emit meta call");

        let expected_subject = gui_state_doc.hash(false).unwrap();
        assert_eq!(emit_meta_call.subject, FixedBytes::from(expected_subject));

        let decoded = RainMetaDocumentV1Item::cbor_decode(emit_meta_call.meta.as_ref()).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].magic, KnownMagic::DotrainSourceV1);
        assert_eq!(decoded[0].payload.as_ref(), dotrain.as_bytes());
    }

    #[test]
    fn test_try_into_emit_meta_call_without_meta() {
        let args = AddOrderArgs {
            dotrain: "".into(),
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
            meta: None,
        };

        assert!(args.try_into_emit_meta_call().unwrap().is_none());
    }

    #[test]
    fn test_try_into_emit_meta_call_without_gui_state_doc() {
        let dotrain = "another dotrain".to_string();
        let rainlang_doc = RainMetaDocumentV1Item {
            payload: ByteBuf::from(dotrain.as_bytes()),
            magic: KnownMagic::RainlangSourceV1,
            content_type: ContentType::OctetStream,
            content_encoding: ContentEncoding::None,
            content_language: ContentLanguage::None,
        };

        let args = AddOrderArgs {
            dotrain,
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
            meta: Some(vec![rainlang_doc]),
        };

        assert!(args.try_into_emit_meta_call().unwrap().is_none());
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
        let result =
            AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment.clone(), None)
                .await
                .unwrap();

        let post_action = result.compose_addorder_post_task().unwrap();

        assert_eq!(post_action, "/* 0. handle-add-order */ \n_ _: 0 0;");
    }

    #[tokio::test]
    async fn test_compose_addorder_post_task_empty_dotrain() {
        let local_evm = LocalEvm::new().await;
        let deployment = get_deployment(&local_evm.url(), *local_evm.deployer.address());
        let result = AddOrderArgs::new_from_deployment("".to_string(), deployment.clone(), None)
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
            None,
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
        AddOrderArgs::new_from_deployment(dotrain, deployment, None)
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
        AddOrderArgs::new_from_deployment(dotrain, deployment, None)
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
        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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
        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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
        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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
                        ReadableClientError::RpcTransportKindError(_)
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
        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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
            meta: None,
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
            meta: None,
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
            meta: None,
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
            inputs: vec![IOV2 {
                token: *local_evm.tokens[0].address(),
                vaultId: B256::from(U256::from(2)),
            }],
            outputs: vec![IOV2 {
                token: *local_evm.tokens[1].address(),
                vaultId: B256::from(U256::from(4)),
            }],
            deployer: *local_evm.deployer.address(),
            bindings: HashMap::new(),
            meta: None,
        };

        let add_order_call = addOrder3Call {
            config: OrderConfigV4 {
                evaluable: EvaluableV4 {
                    interpreter: *local_evm.interpreter.address(),
                    store: *local_evm.store.address(),
                    bytecode: Bytes::from_str(
                        "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000",
                    )
                    .unwrap(),
                },
                validInputs: vec![IOV2 {
                    token: *local_evm.tokens[0].address(),
                    vaultId: B256::from(U256::from(2)),
                }],
                validOutputs: vec![IOV2 {
                    token: *local_evm.tokens[1].address(),
                    vaultId: B256::from(U256::from(4)),
                }],
                nonce: alloy::primitives::private::rand::random::<U256>().into(),
                secret: alloy::primitives::private::rand::random::<U256>().into(),
                meta: Bytes::from_str("0xff0a89c674ee7874a30058382f2a20302e2063616c63756c6174652d696f202a2f200a5f205f3a203020303b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d").unwrap(),
            },
            tasks: vec![
                TaskV2 {
                    evaluable: EvaluableV4 {
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
                max_priority_fee_per_gas: Some(100),
                max_fee_per_gas: Some(200),
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
        assert_eq!(res.call.config.meta.len(), add_order_call.config.meta.len());
        assert_eq!(res.call.tasks, add_order_call.tasks);
        assert_eq!(res.address, *local_evm.orderbook.address());
        assert_eq!(res.max_priority_fee_per_gas, Some(100));
        assert_eq!(res.max_fee_per_gas, Some(200));
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
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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

        let expected_bytes: Bytes = addOrder3Call {
            config: OrderConfigV4 {
                evaluable: EvaluableV4 {
                    interpreter: *local_evm.interpreter.address(),
                    store: *local_evm.store.address(),
                    bytecode: Bytes::from_str("0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000").unwrap(),
                },
                validInputs: vec![
                    IOV2 {
                        token: Address::default(),
                        vaultId: B256::from(U256::from(2)),
                    },
                    IOV2 {
                        token: Address::default(),
                        vaultId: B256::from(U256::from(1)),
                    },
                ],
                validOutputs: vec![IOV2 {
                    token: Address::default(),
                    vaultId: B256::from(U256::from(4)),
                }],
                nonce: U256::from(0).into(),
                secret: U256::from(0).into(),
                meta: Bytes::from_str("0xff0a89c674ee7874a30058382f2a20302e2063616c63756c6174652d696f202a2f200a5f205f3a203020303b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d").unwrap(),
            },
            tasks: vec![TaskV2 {
                evaluable: EvaluableV4 {
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
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment, None)
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
                        ReadableClientError::RpcTransportKindError(_)
                    )
            ),
            "unexpected error variant: {err:?}"
        );
    }

    #[test]
    fn test_try_generate_meta_with_dotrain_instance() {
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

        let dotrain_instance_data = DotrainGuiStateV1 {
            dotrain_hash: B256::from_slice(&[42u8; 32]),
            field_values: BTreeMap::from([(
                "amount".to_string(),
                rain_metadata::types::dotrain::gui_state_v1::ValueCfg {
                    id: "amount_field".to_string(),
                    name: Some("Amount".to_string()),
                    value: "100".to_string(),
                },
            )]),
            deposits: BTreeMap::from([(
                "deposit1".to_string(),
                rain_metadata::types::dotrain::gui_state_v1::ValueCfg {
                    id: "deposit1_field".to_string(),
                    name: Some("Deposit 1".to_string()),
                    value: "1000".to_string(),
                },
            )]),
            select_tokens: BTreeMap::from([(
                "token1".to_string(),
                rain_metadata::types::dotrain::gui_state_v1::ShortenedTokenCfg {
                    network: "ethereum".to_string(),
                    address: Address::default(),
                },
            )]),
            vault_ids: BTreeMap::from([
                ("input_0".to_string(), Some("vault_123".to_string())),
                ("output_0".to_string(), None),
            ]),
            selected_deployment: "test_deployment".to_string(),
        };

        let args = AddOrderArgs {
            dotrain: "".into(),
            inputs: vec![],
            outputs: vec![],
            bindings: HashMap::new(),
            deployer: Address::default(),
            meta: Some(vec![dotrain_instance_data.try_into().unwrap()]),
        };

        let meta_bytes = args.try_generate_meta(dotrain_body).unwrap();

        // Verify that we can decode the meta documents
        let decoded_docs = RainMetaDocumentV1Item::cbor_decode(&meta_bytes).unwrap();
        assert_eq!(decoded_docs.len(), 2);

        // First should be RainlangSourceV1
        assert_eq!(decoded_docs[0].magic, KnownMagic::RainlangSourceV1);

        // Second should be DotrainGuiStateV1
        assert_eq!(decoded_docs[1].magic, KnownMagic::DotrainGuiStateV1);
        assert_eq!(decoded_docs[1].content_type, ContentType::OctetStream);
    }
}
