use alloy::primitives::Bytes;
use rain_orderbook_app_settings::{Config, ParseConfigSourceError};
use rain_orderbook_common::{
    add_order::{AddOrderArgs, AddOrderArgsError},
    frontmatter::parse_frontmatter,
    remove_order::{RemoveOrderArgs, RemoveOrderArgsError},
    transaction::TransactionArgs,
};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AddOrderCalldata(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(AddOrderCalldata);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct RemoveOrderCalldata(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(RemoveOrderCalldata);

/// Represents all possible errors of this module
#[derive(Debug, Error)]
pub enum Error {
    #[error("Undefined deployment")]
    UndefinedDeployment,
    #[error(transparent)]
    ParseConfigSourceError(#[from] ParseConfigSourceError),
    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
    #[error(transparent)]
    RemoveOrderArgsError(#[from] RemoveOrderArgsError),
}

impl Error {
    fn to_readable_msg(&self) -> String {
        match self {
            Self::UndefinedDeployment => {
                "The specified deployment was not found in the .rain file.".to_string()
            }
            Self::ParseConfigSourceError(e) => {
                format!("Failed to parse yaml configuration: {}", e)
            }
            Self::AddOrderArgsError(e) => {
                format!("Failed to prepare the add order calldata: {}", e)
            }
            Self::RemoveOrderArgsError(e) => {
                format!("Failed to prepare the remove order calldata: {}", e)
            }
        }
    }
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<Error> for WasmEncodedError {
    fn from(value: Error) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

/// Get addOrder() calldata from a given dotrain text and deployment key from its frontmatter
#[wasm_export(
    js_name = "getAddOrderCalldata",
    unchecked_return_type = "AddOrderCalldata"
)]
pub async fn get_add_order_calldata(
    dotrain: &str,
    deployment: &str,
) -> Result<AddOrderCalldata, Error> {
    let config: Config = parse_frontmatter(dotrain.to_string()).await?.try_into()?;
    let deployment_ref = config
        .deployments
        .get(deployment)
        .ok_or(Error::UndefinedDeployment)?;
    let add_order_args =
        AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment_ref.deref().clone())
            .await?;

    let tx_args = TransactionArgs {
        rpc_url: deployment_ref.scenario.deployer.network.rpc.to_string(),
        ..Default::default()
    };
    let calldata = add_order_args.get_add_order_calldata(tx_args).await?;
    Ok(AddOrderCalldata(Bytes::copy_from_slice(&calldata)))
}

/// Get removeOrder() calldata for a given order
#[wasm_export(
    js_name = "getRemoveOrderCalldata",
    unchecked_return_type = "RemoveOrderCalldata"
)]
pub async fn get_remove_order_calldata(order: SgOrder) -> Result<RemoveOrderCalldata, Error> {
    let remove_order_args = RemoveOrderArgs { order };
    let calldata = remove_order_args.get_rm_order_calldata().await?;
    Ok(RemoveOrderCalldata(Bytes::copy_from_slice(&calldata)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use alloy::{
            primitives::{Address, FixedBytes, U256},
            sol_types::SolCall,
        };
        use rain_orderbook_bindings::IOrderBookV4::{removeOrder2Call, EvaluableV3, OrderV3, IO};
        use rain_orderbook_subgraph_client::types::common::{SgBigInt, SgBytes, SgOrderbook};
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_get_remove_order_calldata() {
            let remove_order_call = removeOrder2Call {
                order: OrderV3 {
                    owner: Address::from_str("0x6171c21b2e553c59a64d1337211b77c367cefe5d").unwrap(),
                    evaluable: EvaluableV3 {
                        interpreter: Address::from_str(
                            "0x379b966dc6b117dd47b5fc5308534256a4ab1bcc",
                        )
                        .unwrap(),
                        store: Address::from_str("0x6e4b01603edbda617002a077420e98c86595748e")
                            .unwrap(),
                        bytecode: Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000100000000")
                            .unwrap(),
                    },
                    validInputs: vec![IO {
                        token: Address::from_str("0x50c5725949a6f0c72e6c4a641f24049a917db0cb")
                            .unwrap(),
                        decimals: 18,
                        vaultId: U256::from(1),
                    }],
                    validOutputs: vec![IO {
                        token: Address::from_str("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913")
                            .unwrap(),
                        decimals: 6,
                        vaultId: U256::from(1),
                    }],
                    nonce: FixedBytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap()
                },
                tasks: vec![],
            }
            .abi_encode();
            let expected_calldata = Bytes::copy_from_slice(&remove_order_call);

            let order = SgOrder {
                id: SgBytes("1".into()),
                order_bytes: SgBytes("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000006171c21b2e553c59a64d1337211b77c367cefe5d00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000379b966dc6b117dd47b5fc5308534256a4ab1bcc0000000000000000000000006e4b01603edbda617002a077420e98c86595748e000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000950000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000050c5725949a6f0c72e6c4a641f24049a917db0cb000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda0291300000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001".into()),
                order_hash: SgBytes("".into()),
                add_events: vec![],
                timestamp_added: SgBigInt("0".into()),
                owner: SgBytes("".into()),
                active: true,
                inputs: vec![],
                outputs: vec![],
                meta: None,
                orderbook: SgOrderbook {
                    id: SgBytes("1".into()),
                },
                trades: vec![],
                remove_events: vec![],
            };

            let calldata = get_remove_order_calldata(order).await.unwrap();

            assert_eq!(calldata.0, expected_calldata);
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use alloy::{
            hex::encode_prefixed,
            primitives::{Address, B256, U256},
            sol_types::SolValue,
        };
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use rain_orderbook_bindings::IOrderBookV4::IO;
        use std::{collections::HashMap, str::FromStr};

        fn get_dotrain(rpc_url: &str) -> String {
            format!(
                r#"
networks:
  mainnet:
    rpc: {rpc_url}
    chain-id: 1
subgraphs:
  mainnet: https://mainnet-subgraph.com
metaboards:
  mainnet: https://mainnet-metaboard.com
orderbooks:
  mainnet:
    address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    network: mainnet
    subgraph: mainnet
tokens:
  token1:
    network: mainnet
    address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    decimals: 18
    label: Wrapped Ether
    symbol: WETH
  token2:
    network: mainnet
    address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    decimals: 6
    label: USD Coin
    symbol: USDC
deployers:
  scenario1:
    address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    network: mainnet
orders:
  order1:
    deployer: scenario1
    orderbook: mainnet
    inputs:
      - token: token1
        vault-id: 1
    outputs:
      - token: token2
        vault-id: 2
scenarios:
  scenario1:
    bindings:
      key1: 10
    scenarios:
      scenario2:
        bindings:
          key2: 20
        runs: 10
deployments:
  deployment1:
    order: order1
    scenario: scenario1.scenario2
---
#key1 !Test binding
#key2 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#
            )
        }

        #[tokio::test]
        async fn test_get_add_order_calldata() {
            let rpc_server = MockServer::start_async().await;
            let dotrain = get_dotrain(&rpc_server.url("/rpc"));

            rpc_server.mock(|when, then| {
                when.path("/rpc").body_contains("0xf0cfdd37");
                then.body(
                    Response::new_success(
                        1,
                        &B256::left_padding_from(
                            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
                                .unwrap()
                                .as_slice(),
                        )
                        .to_string(),
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });
            rpc_server.mock(|when, then| {
                when.path("/rpc").body_contains("0xc19423bc");
                then.body(
                    Response::new_success(
                        2,
                        &B256::left_padding_from(
                            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
                                .unwrap()
                                .as_slice(),
                        )
                        .to_string(),
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });
            rpc_server.mock(|when, then| {
                when.path("/rpc").body_contains("0x24376855");
                then.body(
                    Response::new_success(
                        3,
                        &B256::left_padding_from(
                            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
                                .unwrap()
                                .as_slice(),
                        )
                        .to_string(),
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });
            rpc_server.mock(|when, then| {
                when.path("/rpc").body_contains("0xa3869e14");
                then.body(
                    Response::new_success(
                        4,
                        &encode_prefixed(Bytes::from(vec![1, 2]).abi_encode()),
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });

            let expected_calldata: Bytes = AddOrderArgs {
                dotrain: dotrain.clone(),
                inputs: vec![IO {
                    token: Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
                    decimals: 18,
                    vaultId: U256::from(1),
                }],
                outputs: vec![IO {
                    token: Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
                    decimals: 6,
                    vaultId: U256::from(2),
                }],
                deployer: Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
                bindings: HashMap::from([
                    ("key1".to_string(), "10".to_string()),
                    ("key2".to_string(), "20".to_string()),
                ]),
            }
            .get_add_order_calldata(TransactionArgs {
                rpc_url: rpc_server.url("/rpc"),
                ..Default::default()
            })
            .await
            .unwrap()
            .into();

            let calldata = get_add_order_calldata(&dotrain, "deployment1")
                .await
                .unwrap();

            // Nonce and secret are random, so we can't compare the whole calldata
            assert_eq!(calldata.0[..164], expected_calldata[..164]);
            assert_eq!(calldata.0[228..], expected_calldata[228..]);
        }

        #[tokio::test]
        async fn test_get_add_order_calldata_invalid_deployment() {
            let rpc_server = MockServer::start_async().await;
            let dotrain = get_dotrain(&rpc_server.url("/rpc"));

            let err = get_add_order_calldata(&dotrain, "invalid-deployment")
                .await
                .unwrap_err();
            assert!(matches!(err, Error::UndefinedDeployment));
        }

        #[tokio::test]
        async fn test_get_add_order_calldata_invalid_dotrain() {
            let err = get_add_order_calldata(&"invalid-dotrain".to_string(), "deployment1")
                .await
                .unwrap_err();
            assert!(matches!(err, Error::UndefinedDeployment));

            let err = get_add_order_calldata(
                &r#"
deployments:
  deployment1:
    order: order1
    scenario: scenario1
---
"#
                .to_string(),
                "deployment1",
            )
            .await
            .unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, Error::ParseConfigSourceError(_)));
        }

        #[tokio::test]
        async fn test_get_add_order_calldata_missing_rpc_data() {
            let rpc_server = MockServer::start_async().await;
            let dotrain = get_dotrain(&rpc_server.url("/rpc"));

            let err = get_add_order_calldata(&dotrain, "deployment1")
                .await
                .unwrap_err();
            assert!(matches!(
                err,
                Error::AddOrderArgsError(AddOrderArgsError::DISPairError(_))
            ));
        }
    }
}
