use crate::{
    dotrain_add_order_lsp::LANG_SERVICES,
    transaction::{TransactionArgs, TransactionArgsError},
};
use alloy_ethers_typecast::transaction::{
    ReadContractParameters, ReadableClientError, ReadableClientHttp, WritableClientError,
    WriteTransaction, WriteTransactionStatus,
};
use alloy_primitives::{hex::FromHexError, Address, U256};
use alloy_sol_types::SolCall;
use dotrain::{error::ComposeError, RainDocument, Rebind};
use rain_interpreter_dispair::{DISPair, DISPairError};
use rain_interpreter_parser::{Parser, ParserError, ParserV1};
use rain_metadata::{
    ContentEncoding, ContentLanguage, ContentType, Error as RainMetaError, KnownMagic,
    RainMetaDocumentV1Item,
};
use rain_orderbook_app_settings::deployment::Deployment;
use rain_orderbook_bindings::{
    IOrderBookV3::{addOrderCall, EvaluableConfigV3, OrderConfigV2, IO},
    ERC20::decimalsCall,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use thiserror::Error;

pub static ORDERBOOK_ORDER_ENTRYPOINTS: [&str; 2] = ["calculate-io", "handle-io"];

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
    ) -> Result<(Vec<u8>, Vec<U256>), AddOrderArgsError> {
        let client = ReadableClientHttp::new_from_url(rpc_url)
            .map_err(AddOrderArgsError::ReadableClientError)?;
        let dispair = DISPair::from_deployer(self.deployer, client.clone())
            .await
            .map_err(AddOrderArgsError::DISPairError)?;

        let parser: ParserV1 = dispair.clone().into();
        let rainlang_parsed = parser
            .parse_text(rainlang.as_str(), client)
            .await
            .map_err(AddOrderArgsError::ParserError)?;

        Ok((rainlang_parsed.bytecode, rainlang_parsed.constants))
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
        // Parse file into dotrain document
        let meta_store = LANG_SERVICES.meta_store();

        let mut rebinds = None;
        if !self.bindings.is_empty() {
            rebinds = Some(
                self.bindings
                    .iter()
                    .map(|(key, value)| Rebind(key.clone(), value.clone()))
                    .collect(),
            );
        };
        let dotrain_doc =
            RainDocument::create(self.dotrain.clone(), Some(meta_store), None, rebinds);

        Ok(dotrain_doc.compose(&ORDERBOOK_ORDER_ENTRYPOINTS)?)
    }

    /// Generate an addOrder call from given dotrain
    async fn try_into_call(&self, rpc_url: String) -> Result<addOrderCall, AddOrderArgsError> {
        let rainlang = self.compose_to_rainlang()?;
        let (bytecode, constants) = self.try_parse_rainlang(rpc_url, rainlang.clone()).await?;
        let meta = self.try_generate_meta(rainlang)?;

        Ok(addOrderCall {
            config: OrderConfigV2 {
                validInputs: self.inputs.clone(),
                validOutputs: self.outputs.clone(),
                evaluableConfig: EvaluableConfigV3 {
                    deployer: self.deployer,
                    bytecode,
                    constants,
                },
                meta,
            },
        })
    }

    pub async fn execute<S: Fn(WriteTransactionStatus<addOrderCall>)>(
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
        let deployer = Address::random();
        let bytecode = vec![
            0x60, 0x60, 0x60, 0x40, 0x60, 0x60, 0x60, 0x40, 0x60, 0x60, 0x60, 0x40, 0x60, 0x60,
            0x60, 0x40,
        ];
        let constants = vec![U256::from(0), U256::from(1), U256::from(2), U256::from(3)];
        let meta = vec![9, 10, 11, 12];

        let order_config_v2 = OrderConfigV2 {
            validInputs: inputs.clone(),
            validOutputs: outputs.clone(),
            evaluableConfig: EvaluableConfigV3 {
                deployer,
                bytecode: bytecode.clone(),
                constants: constants.clone(),
            },
            meta: meta.clone(),
        };

        assert_eq!(order_config_v2.validInputs, inputs);
        assert_eq!(order_config_v2.validOutputs, outputs);
        assert_eq!(order_config_v2.evaluableConfig.deployer, deployer);
        assert_eq!(order_config_v2.evaluableConfig.bytecode, bytecode);
        assert_eq!(order_config_v2.evaluableConfig.constants, constants);
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
}
