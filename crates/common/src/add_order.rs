use crate::{
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
use rain_interpreter_parser::{Parser2, ParserError, ParserV2};
use rain_metadata::{
    ContentEncoding, ContentLanguage, ContentType, Error as RainMetaError, KnownMagic,
    RainMetaDocumentV1Item,
};
use rain_orderbook_app_settings::deployment::Deployment;
use rain_orderbook_bindings::{
    IOrderBookV4::{addOrder2Call, EvaluableV3, OrderConfigV3, TaskV1, IO},
    ERC20::decimalsCall,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use thiserror::Error;

pub static ORDERBOOK_ORDER_ENTRYPOINTS: [&str; 2] = ["calculate-io", "handle-io"];
pub static ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS: [&str; 1] = ["post-add-order"];

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
}

#[derive(Serialize, Deserialize, Clone)]
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
        deployment: Deployment,
    ) -> Result<AddOrderArgs, AddOrderArgsError> {
        let random_vault_id: U256 = rand::random();
        let mut inputs = vec![];
        for input in &deployment.order.inputs {
            if let Some(decimals) = input.token.decimals {
                inputs.push(IO {
                    token: input.token.address,
                    vaultId: input.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            } else {
                let client = ReadableClientHttp::new_from_url(input.token.network.rpc.to_string())?;
                let parameters = ReadContractParameters {
                    address: input.token.address,
                    call: decimalsCall {},
                    block_number: None,
                };
                let decimals = client.read(parameters).await?._0;
                inputs.push(IO {
                    token: input.token.address,
                    vaultId: input.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            }
        }

        let mut outputs = vec![];
        for output in &deployment.order.outputs {
            if let Some(decimals) = output.token.decimals {
                outputs.push(IO {
                    token: output.token.address,
                    vaultId: output.vault_id.unwrap_or(random_vault_id),
                    decimals,
                });
            } else {
                let client =
                    ReadableClientHttp::new_from_url(output.token.network.rpc.to_string())?;
                let parameters = ReadContractParameters {
                    address: output.token.address,
                    call: decimalsCall {},
                    block_number: None,
                };
                let decimals = client.read(parameters).await?._0;
                outputs.push(IO {
                    token: output.token.address,
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rain_orderbook_app_settings::deployer::Deployer;
    use rain_orderbook_app_settings::network::Network;
    use rain_orderbook_app_settings::order::{Order, OrderIO};
    use rain_orderbook_app_settings::scenario::Scenario;
    use rain_orderbook_app_settings::token::Token;
    use rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL;
    use url::Url;

    use super::*;

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
        let network = Network {
            name: "test-network".to_string(),
            rpc: Url::parse(CI_DEPLOY_POLYGON_RPC_URL).unwrap(),
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = Deployer {
            network: network_arc.clone(),
            address: Address::default(),
            label: None,
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = Scenario {
            name: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = Token {
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
        let order = Order {
            inputs: vec![
                OrderIO {
                    token: token1_arc.clone(),
                    vault_id: None,
                },
                OrderIO {
                    token: token2_arc.clone(),
                    vault_id: Some(known_vault_id),
                },
            ],
            outputs: vec![OrderIO {
                token: token3_arc.clone(),
                vault_id: None,
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = Deployment {
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = r#"
some front matter
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#;
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
        let network = Network {
            name: "test-network".to_string(),
            rpc: Url::parse(CI_DEPLOY_POLYGON_RPC_URL).unwrap(),
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = Deployer {
            network: network_arc.clone(),
            address: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba"
                .parse::<Address>()
                .unwrap(),
            label: None,
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = Scenario {
            name: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token3".to_string()),
        };
        let token1_arc = Arc::new(token1);
        let token2_arc = Arc::new(token2);
        let token3_arc = Arc::new(token3);
        let order = Order {
            inputs: vec![
                OrderIO {
                    token: token1_arc.clone(),
                    vault_id: Some(U256::from(2)),
                },
                OrderIO {
                    token: token2_arc.clone(),
                    vault_id: Some(U256::from(1)),
                },
            ],
            outputs: vec![OrderIO {
                token: token3_arc.clone(),
                vault_id: Some(U256::from(4)),
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = Deployment {
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = r#"
some front matter
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
_ _: 0 0;
"#;
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment)
            .await
            .unwrap();

        let add_order_call = result
            .try_into_call(CI_DEPLOY_POLYGON_RPC_URL.to_string())
            .await
            .unwrap();

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
            "0x6352593f4018c99df731de789e2a147c7fb29370"
                .parse::<Address>()
                .unwrap()
        );

        assert_eq!(
            add_order_call.tasks[0].evaluable.store,
            "0xde38ad4b13d5258a5653e530ecdf0ca71b4e8a51"
                .parse::<Address>()
                .unwrap()
        );

        assert_eq!(add_order_call.tasks[0].evaluable.bytecode.len(), 111);
        assert_eq!(add_order_call.tasks[0].signedContext.len(), 0);
    }

    #[tokio::test]
    async fn test_add_order_post_action() {
        let network = Network {
            name: "test-network".to_string(),
            rpc: Url::parse(CI_DEPLOY_POLYGON_RPC_URL).unwrap(),
            chain_id: 137,
            label: None,
            network_id: None,
            currency: None,
        };
        let network_arc = Arc::new(network);
        let deployer = Deployer {
            network: network_arc.clone(),
            address: Address::default(),
            label: None,
        };
        let deployer_arc = Arc::new(deployer);
        let scenario = Scenario {
            name: "test-scenario".to_string(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            deployer: deployer_arc.clone(),
        };
        let token1 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token1".to_string()),
        };
        let token2 = Token {
            address: Address::default(),
            network: network_arc.clone(),
            decimals: Some(18),
            label: None,
            symbol: Some("Token2".to_string()),
        };
        let token3 = Token {
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
        let order = Order {
            inputs: vec![
                OrderIO {
                    token: token1_arc.clone(),
                    vault_id: None,
                },
                OrderIO {
                    token: token2_arc.clone(),
                    vault_id: Some(known_vault_id),
                },
            ],
            outputs: vec![OrderIO {
                token: token3_arc.clone(),
                vault_id: None,
            }],
            network: network_arc.clone(),
            deployer: None,
            orderbook: None,
        };
        let deployment = Deployment {
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };

        let dotrain = r#"
some front matter
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
_ _: 0 0;
"#;
        let result = AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment.clone())
            .await
            .unwrap();

        let post_action = result.compose_addorder_post_task().unwrap();

        assert_eq!(post_action, "/* 0. post-add-order */ \n_ _: 0 0;");
    }
}
