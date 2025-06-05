use crate::error::CommandResult;
use crate::{toast::toast_error, transaction_status::TransactionStatusNoticeRwLock};
use alloy::primitives::Bytes;
use rain_orderbook_app_settings::{deployment::DeploymentCfg, scenario::ScenarioCfg};
use rain_orderbook_common::{
    add_order::AddOrderArgs, csv::TryIntoCsv, dotrain_order::DotrainOrder,
    remove_order::RemoveOrderArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
    types::FlattenError, types::OrderFlattened,
};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};

#[tauri::command]
pub async fn orders_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let orders = subgraph_args
        .to_subgraph_client()?
        .orders_list_all()
        .await?;
    let orders_flattened: Vec<OrderFlattened> = orders
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?;
    let csv_text = orders_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}

#[tauri::command]
pub async fn order_add<R: Runtime>(
    app_handle: AppHandle<R>,
    dotrain: String,
    deployment: DeploymentCfg,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice = TransactionStatusNoticeRwLock::new("Add order".into());
    let add_order_args = AddOrderArgs::new_from_deployment(dotrain, deployment).await?;
    add_order_args
        .execute(transaction_args, |status| {
            tx_status_notice.update_status_and_emit(&app_handle, status);
        })
        .await
        .map_err(|e| {
            toast_error(&app_handle, e.to_string());
            e
        })?;

    Ok(())
}

#[tauri::command]
pub async fn order_remove<R: Runtime>(
    app_handle: AppHandle<R>,
    id: String,
    transaction_args: TransactionArgs,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let order = subgraph_args
        .to_subgraph_client()
        .map_err(|e| {
            toast_error(&app_handle, String::from("Subgraph URL is invalid"));
            e
        })?
        .order_detail(&id.into())
        .await
        .map_err(|e| {
            toast_error(&app_handle, e.to_string());
            e
        })?;
    let remove_order_args: RemoveOrderArgs = order.into();

    let tx_status_notice = TransactionStatusNoticeRwLock::new("Remove order".into());
    let _ = remove_order_args
        .execute(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(&app_handle, status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(&app_handle, e.to_string());
        });

    Ok(())
}

#[tauri::command]
pub async fn order_add_calldata<R: Runtime>(
    app_handle: AppHandle<R>,
    dotrain: String,
    deployment: DeploymentCfg,
    transaction_args: TransactionArgs,
) -> CommandResult<Bytes> {
    let add_order_args = AddOrderArgs::new_from_deployment(dotrain, deployment).await?;
    let calldata = add_order_args
        .get_add_order_calldata(transaction_args)
        .await
        .map_err(|e| {
            toast_error(&app_handle, e.to_string());
            e
        })?;

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn order_remove_calldata<R: Runtime>(
    app_handle: AppHandle<R>,
    id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<Bytes> {
    let order = subgraph_args
        .to_subgraph_client()
        .map_err(|e| {
            toast_error(&app_handle, String::from("Subgraph URL is invalid"));
            e
        })?
        .order_detail(&id.into())
        .await
        .map_err(|e| {
            toast_error(&app_handle, e.to_string());
            e
        })?;
    let remove_order_args: RemoveOrderArgs = order.into();
    let calldata = remove_order_args
        .get_rm_order_calldata()
        .await
        .map_err(|e| {
            toast_error(&app_handle, e.to_string());
            e
        })?;

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn compose_from_scenario(
    dotrain: String,
    settings: Option<Vec<String>>,
    scenario: ScenarioCfg,
) -> CommandResult<String> {
    let mut dotrain_order = DotrainOrder::new();
    dotrain_order.initialize(dotrain, settings).await?;
    Ok(dotrain_order
        .compose_scenario_to_rainlang(scenario.key)
        .await?)
}

#[tauri::command]
pub async fn validate_spec_version(dotrain: String, settings: Vec<String>) -> CommandResult<()> {
    let mut dotrain_order = DotrainOrder::new();
    dotrain_order.initialize(dotrain, Some(settings)).await?;
    Ok(dotrain_order.validate_spec_version().await?)
}

#[cfg(test)]
mod tests {
    use alloy::hex::ToHex;
    use alloy::primitives::U256;
    use alloy::sol_types::SolCall;
    use dotrain::error::ComposeError;
    use httpmock::MockServer;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::FieldErrorKind;
    use rain_orderbook_bindings::IOrderBookV4::{addOrder2Call, IO};
    use rain_orderbook_common::add_order::AddOrderArgsError;
    use rain_orderbook_common::transaction::TransactionArgsError;
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use strict_yaml_rust::StrictYaml;
    use tauri::Manager;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::error::CommandError;
    use rain_orderbook_app_settings::{deployer::DeployerCfg, yaml::YamlError};
    use rain_orderbook_common::dotrain_order::DotrainOrderError;
    use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;

    #[tokio::test]
    async fn test_orders_list_write_csv_ok() {
        let temp_file = NamedTempFile::new().unwrap();
        let csv_path = temp_file.path().to_path_buf();

        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/sg").body_contains("\"skip\":0");
            then.status(200)
                .body_from_file("../../test-resources/example-subgraph-orders-res1.json");
        });

        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "orders": []
                }
            }));
        });

        // Call the function
        orders_list_write_csv(
            csv_path.clone(),
            SubgraphArgs {
                url: server.url("/sg"),
            },
        )
        .await
        .unwrap();

        // Verify the CSV file was created and contains expected content
        let actual_csv_content = std::fs::read_to_string(csv_path).unwrap();

        let expected_csv_content = r#"
id,timestamp,timestamp_display,owner,order_active,interpreter,interpreter_store,transaction,valid_inputs_vaults,valid_outputs_vaults,valid_inputs_token_symbols_display,valid_outputs_token_symbols_display,trades
0xce594506aded89c9911e6d25d335737c1593d5fcba0d5085f640ad33608900d4,1746708958,2025-05-08 12:55:58 UTC,0x627a12ce1f6d42c9305e03e83fe044e8c3c1a32c,true,bd8849759749b4d8506bc851acef0e19f34eabee,8d96ea3ef24d7123882c51ce4325b89bc0d63f9e,0xb4633e1c36ba079749df9f6c3be740f44622ed34fc7457d1dbe1a77b9608f4d7,"90191762620663887453907314496784028792739187795829076274284824685275793811138, 90191762620663887453907314496784028792739187795829076274284824685275793811138","90191762620663887453907314496784028792739187795829076274284824685275793811138, 90191762620663887453907314496784028792739187795829076274284824685275793811138","NST, BO","NST, BO","0x02656cc86909041942571c0c5c153151eb3f366ea2efd9cd5aeaca7fa5442db3, 0x17acda643769e9d2246c01438b8c380c7cebedceca80015642ab005387f48053, 0x17b14cd0c1f001d944b2be134dc70647821a5eaa9b2187c473d78c3afd7e3e34, 0x1db3570a53156b8c001a7d48a461045be0194761d3a87a0126f3677e900c7eb4, 0x1dc167c6d972e160bc5feddba5283bddc99fffb3e9148b26eaa700511b4a6b8d, 0x2369af92982660a4b77adf1246388feaf75007a25e601ca9bc891deafa4d127c, 0x30b68db2fcaa9ec3883dc0566e056f18241dcf946e7b4c6e1d1551991c2b7cb8, 0x3a6e46d564a65af7c5cd752a6b08b26cb1a466af14c2b767d0d4deeea9b40f4c, 0x45ba490ee63d1503332be96fa5b718cfbfe16337db9f73bff56f7bf506b1c479, 0x4ac5b430259aac08a305dab67c7b8be2e75fe9138a64dfbc3fce25698d6fdfee, 0x51afee427fe2c5096dfd1b9d9d344945c856b57c174a719aa2a53d43fd287a93, 0x55f29d5f6a6f49ac51347a5a6ed7b29db89a84493cbd5eaedafd5c0238031e48, 0x7eab8cf4d3ed74e3d94c2ff366525fa0210920153ff5e927485fa108f4142512, 0x8494208aba85005d816a1bfb94114045b636b6ba113c7d3fa4984d706f46e339, 0x88eaf330f2824a48f3fa7b62f60474d59a93d3168dd141c0d0a3a39fe45cf3af, 0x9a280ea919af7f0ff89d75d32c9fb2d292ef86cb79a9531f242c98ae79926b70, 0x9caeb42f2d2c3acd71959de8662283e008e079592bd6d3a79e92e344e0c51c90, 0xa22ce8b517ef1919b89f4f0ee7957e22a85ad28e03c13ef21a6c76766d104a48, 0xa379a39c24d2e43f98ff3b6b2fee1a05f336c08c4c4b3b01da25587c7199f680, 0xac4b1b6ac7e8c4f8b5abb19dd6af9d566d06ad60493665caa5c282c2a4a10a63, 0xad4eaa40b1c6549fe04fd41659f053b246ce5436522869e522987327ded5c75c, 0xae5d3346a4a1ff1f22cd4443c6a192313295f196970f9b862f4477f57b3c05ca, 0xb746bfbccfffc6b5d43d76a2b9ce3dfaeaf7efe4f79cdcd72433e04f6350ac1d, 0xbd0607e22a3309ae71efb8da38a0c861797d04ce8075ace1a927d8979af033c5, 0xcb84ade9980389fadca6133d56e5adcba44aef3384a1fe6644c4f4b63db29b88, 0xd1799349765b1687d3b49783186c114e16bfac2ff58f9ce663d88426c87f216f, 0xd48bca326a458acaa0729de5cdf8b1c6930a1b4143142cadec3109b87139a3a3, 0xf579b721ae550d20530bb8faa03d1593dd5e0b19a441f1ce5a366901610c53b5, 0xf992bd388c39b5bf0d1e3dacdbaa65222a9fcd9b3d3dc5c0839a891b3b203c4b, 0xfe57129001733375f42b8b4d37f6311ef739bf4357926ccf954e99075bfaaa32"
0xf549990803031db59083d12ae3e51ceab96e09987e33b5e42b10804c7b36179c,1746707888,2025-05-08 12:38:08 UTC,0x627a12ce1f6d42c9305e03e83fe044e8c3c1a32c,true,bd8849759749b4d8506bc851acef0e19f34eabee,8d96ea3ef24d7123882c51ce4325b89bc0d63f9e,0x980adbf12c92443e1118d57b3922cd3338b545d4af4be8ac5cab00a66ab17cfb,"57232934479656398024826646079073261177566950623391670149778471144032687355733, 57232934479656398024826646079073261177566950623391670149778471144032687355733","57232934479656398024826646079073261177566950623391670149778471144032687355733, 57232934479656398024826646079073261177566950623391670149778471144032687355733","NST, Boop","NST, Boop",0xe0924f238d9a451c0ed12d61128dbe924a7c7d627ab6be997bce0d9e2521bfa6
"#.trim_start();

        assert_eq!(actual_csv_content, expected_csv_content);
    }

    #[tokio::test]
    async fn test_orders_list_write_csv_err() {
        let temp_file = NamedTempFile::new().unwrap();
        let csv_path = temp_file.path().to_path_buf();

        let err = orders_list_write_csv(
            csv_path.clone(),
            SubgraphArgs {
                url: "invalid-url".to_string(),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::URLParseError(url::ParseError::RelativeUrlWithoutBase)
        ));

        let server = MockServer::start();
        server.mock(|when, then| {
            when.path("/sg");
            then.status(500).body("Internal Server Error");
        });

        let err = orders_list_write_csv(
            csv_path.clone(),
            SubgraphArgs {
                url: server.url("/sg"),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::OrderbookSubgraphClientError(
                OrderbookSubgraphClientError::CynicClientError(_)
            )
        ));
    }

    // NOTE: unimplemented due to order_add depending on Ledger
    // #[tokio::test]
    // async fn test_order_add_ok() { }

    #[tokio::test]
    async fn test_order_add_err() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let dotrain = r#"
networks:
    sepolia:
        rpc: http://example.com
        chain-id: 0
deployers:
    sepolia:
        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#
        .to_string();

        let deployment = DeploymentCfg::default();
        let transaction_args = TransactionArgs::default();

        let err = order_add(app_handle, dotrain, deployment, transaction_args)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::AddOrderArgsError(AddOrderArgsError::TransactionArgs(
                TransactionArgsError::ChainIdNone
            ))
        ));
    }

    #[tokio::test]
    async fn test_order_remove_ok() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let subgraph_server = MockServer::start();

        subgraph_server.mock(|when, then| {
            when.path("/sg");
            then.status(200)
                .body_from_file("../../test-resources/example-subgraph-order-res1.json");
        });

        let subgraph_args = SubgraphArgs {
            url: subgraph_server.url("/sg"),
        };

        let rpc_server = MockServer::start();

        rpc_server.mock(|when, then| {
            when.path("/rpc");
            then.status(200);
        });

        let transaction_args = TransactionArgs {
            rpc_url: rpc_server.url("/rpc"),
            ..Default::default()
        };

        order_remove(
            app_handle,
            "0x123".to_string(),
            transaction_args,
            subgraph_args,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_order_remove_err() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let subgraph_args = SubgraphArgs {
            url: "invalid-url".to_string(),
        };

        let err = order_remove(
            app_handle,
            "0x123".to_string(),
            TransactionArgs::default(),
            subgraph_args,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::URLParseError(url::ParseError::RelativeUrlWithoutBase)
        ));
    }

    #[tokio::test]
    async fn test_order_add_calldata_ok() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1 = *local_evm.tokens[0].address();
        let token2 = *local_evm.tokens[1].address();

        let dotrain = format!(
            r#"
version: {spec_version}
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
        address: {token1}
        decimals: 18
        label: Token2
        symbol: Token2
    t2:
        network: some-key
        address: {token2}
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
              vault-id: 0x01
        outputs:
            - token: t2
              vault-id: 0x02
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
            spec_version = SpecVersion::current(),
        );

        let mut order = DotrainOrder::new();
        order.initialize(dotrain.clone(), None).await.unwrap();
        let deployment = order.dotrain_yaml().get_deployment("some-key").unwrap();

        let transaction_args = TransactionArgs {
            orderbook_address: *orderbook.address(),
            rpc_url: local_evm.url(),
            ..Default::default()
        };

        let calldata = order_add_calldata(app_handle, dotrain, deployment, transaction_args)
            .await
            .unwrap();

        let decoded = addOrder2Call::abi_decode(&calldata, true).unwrap();

        assert_eq!(
            decoded.config.validInputs,
            vec![IO {
                token: token1,
                decimals: 18,
                vaultId: U256::from(0x01),
            }]
        );
        assert_eq!(
            decoded.config.validOutputs,
            vec![IO {
                token: token2,
                decimals: 18,
                vaultId: U256::from(0x02),
            }]
        );
    }

    #[tokio::test]
    async fn test_order_add_calldata_err() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc");
            then.status(500).body("Internal Server Error");
        });

        let dotrain = "invalid-dotrain".to_string();
        let deployment = DeploymentCfg::default();
        let transaction_args = TransactionArgs::default();

        let err = order_add_calldata(app_handle, dotrain, deployment.clone(), transaction_args)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::AddOrderArgsError(AddOrderArgsError::ComposeError(ComposeError::Problems(problems)))
            if problems.len() == 2
                && problems[0].msg == "cannot find front matter splitter"
                && problems[1].msg == "unexpected token"
        ));
    }

    #[tokio::test]
    async fn test_order_remove_calldata_ok() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/sg");
            then.status(200)
                .body_from_file("../../test-resources/example-subgraph-order-res1.json");
        });

        let calldata = order_remove_calldata(
            app_handle,
            "0x8505ec2e04a958dd8fae1df9df1675d84ec2a29f994cea49351f97fed3f455f7".to_string(),
            SubgraphArgs {
                url: server.url("/sg"),
            },
        )
        .await
        .unwrap();

        let expected = "8d7b6beb00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000780000000000000000000000000ca977a85327dd36494d57ff11ca3be02af1142aa00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006c0198da998908ba92dd94c43e0ffba5cb2f3f4e858fb92fd9fbc09db20a5cd84350000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae6000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000005030000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000001f444dd9fb80d80000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d7573656400000000000000000000000000000000000000000000000000000000000000000000000000000000000000d8d726b7177a80000000000000000000000000000000000000000000000000001043561a88293000000000000000000000000000000000000000000000000000c328093e61ee400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000018650127cc3dc80000000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e800000000000000000000000000000000000000000000000000000100c30cc5b443a0000000000000000000000000000000000000000000000000010729fa58404bd600000000000000000000000000000000000000000000000000000000000002830b00000024007400e0015801b401e001f40218025c0264080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100004011000002e12000001100006011000052e120000001000053d12000001100005001000042e1200000010000601100006001000032e120000481200011d0b020a0010000001100004011000072713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000800100001201200001f12000001100000011000094712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b1000090110000a001000013d1200001b12000001100007001000013d1200000b10000a001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a1000000110000b031000010c1200004a020000001000000110000c031000010c1200004a020000040200010110000d031000010c12000049110000080300020110000b031000010c120000491100000110000c031000010c12000049110000100c01030110000f001000002e1200000110000e3e120000001000010010000100100001001000010010000100100001001000010010000100100001001000013d1a0000010100010110001001010001011000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000019831cfb53a0dbead9866c43557c1d48dff765670000000000000000000000000000000000000000000000000000000000000012c8c19c32ec9a5dd4bc5d1299c0e59489500f8c098d64293ee33208d0b7e44108000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012c8c19c32ec9a5dd4bc5d1299c0e59489500f8c098d64293ee33208d0b7e441080000000000000000000000000000000000000000000000000000000000000000";

        assert_eq!(calldata.encode_hex::<String>(), expected.to_string());
    }

    #[tokio::test]
    async fn test_order_remove_calldata_err() {
        let mock_app = tauri::test::mock_app();
        let app_handle = mock_app.app_handle();

        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/sg");
            then.status(500).body("Internal Server Error");
        });

        let err = order_remove_calldata(
            app_handle,
            "0xce594506aded89c9911e6d25d335737c1593d5fcba0d5085f640ad33608900d4".to_string(),
            SubgraphArgs {
                url: server.url("/sg"),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::OrderbookSubgraphClientError(
                OrderbookSubgraphClientError::CynicClientError(_)
            )
        ));
    }

    #[tokio::test]
    async fn test_compose_from_scenario_ok() {
        let server = MockServer::start();
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
            spec_version = SpecVersion::current(),
        );

        let actual_rainlang = compose_from_scenario(
            dotrain.clone(),
            None,
            ScenarioCfg {
                document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
                key: "polygon".to_string(),
                bindings: HashMap::new(),
                runs: None,
                blocks: None,
                deployer: Arc::new(DeployerCfg::default()),
            },
        )
        .await
        .unwrap();

        let expected_rainlang = "/* 0. calculate-io */ \n_ _: 0 0;\n\n/* 1. handle-io */ \n:;";
        assert_eq!(actual_rainlang, expected_rainlang);
    }

    #[tokio::test]
    async fn test_compose_from_scenario_err() {
        let server = MockServer::start();
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
            spec_version = SpecVersion::current(),
        );

        // Test scenario not found error
        let err = compose_from_scenario(
            dotrain.clone(),
            None,
            ScenarioCfg {
                document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
                key: "nonexistent".to_string(),
                bindings: HashMap::new(),
                runs: None,
                blocks: None,
                deployer: Arc::new(DeployerCfg::default()),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::DotrainOrderError(DotrainOrderError::YamlError(YamlError::KeyNotFound(key)))
            if key == "nonexistent"
        ));

        // Test compose error with invalid rainlang
        let dotrain_invalid = format!(
            r#"
version: {spec_version}
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: invalid syntax;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
            spec_version = SpecVersion::current(),
        );

        let err = compose_from_scenario(
            dotrain_invalid,
            None,
            ScenarioCfg {
                document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
                key: "polygon".to_string(),
                bindings: HashMap::new(),
                runs: None,
                blocks: None,
                deployer: Arc::new(DeployerCfg::default()),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::DotrainOrderError(DotrainOrderError::ComposeError(_))
        ));
    }

    #[tokio::test]
    async fn test_validate_spec_version_ok() {
        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    sepolia:
        rpc: http://example.com
        chain-id: 0
deployers:
    sepolia:
        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
---
#calculate-io
_ _: 0 0;
#handle-io
:;
"#,
            spec_version = SpecVersion::current(),
        );

        validate_spec_version(dotrain.to_string(), vec![])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_validate_spec_version_err() {
        let dotrain = r#"
networks:
    sepolia:
        rpc: http://example.com
        chain-id: 0
deployers:
    sepolia:
        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#;

        let err = validate_spec_version(dotrain.to_string(), vec![])
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::DotrainOrderError(DotrainOrderError::YamlError(YamlError::Field {
                kind: FieldErrorKind::Missing(ref s),
                location: ref loc
            })) if s == "version" && loc == "root"
        ));

        let dotrain = r#"
version: 2
networks:
    sepolia:
        rpc: http://example.com
        chain-id: 0
deployers:
    sepolia:
        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
---
#calculate-io
_ _: 0 0;
#handle-io
:;"#;

        let err = validate_spec_version(dotrain.to_string(), vec![])
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::DotrainOrderError(DotrainOrderError::SpecVersionMismatch(ref expected, ref actual))
            if expected == "1" && actual == "2"
        ));
    }
}
