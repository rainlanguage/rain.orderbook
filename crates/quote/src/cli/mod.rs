use crate::{OrderQuoteValue, QuoteResult};
use alloy::primitives::Address;
use clap::{command, ArgAction, Parser};
use serde::{Deserialize, Serialize};
use std::{fs::write, io::Write, path::PathBuf};
use url::Url;

mod input;
pub use input::*;

/// Rain orderbook Quoter CLI app entrypoint sruct
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about = "Rain Orderbook Quote CLI", long_about = None)]
pub struct Quoter {
    // input group, only one of which can be specified at a time
    #[command(flatten)]
    pub input: Input,

    /// RPC URL of the evm chain to quote
    #[arg(short, long, env, value_name = "URL", hide_env_values = true)]
    pub rpc: Url,

    /// Subgraph URL to read orders details from, presence of this
    /// arg determines what type input's undelying content should be in
    #[arg(
        short,
        long,
        env,
        value_name = "URL",
        visible_alias = "sg",
        hide_env_values = true
    )]
    pub subgraph: Option<Url>,

    /// Optional block number to quote at
    #[arg(short, long, env, value_name = "INTEGER")]
    pub block_number: Option<u64>,

    /// Optional multicall3 address to use when quoting
    #[arg(short, long, env, value_name = "ADDRESS")]
    pub multicall_address: Option<Address>,

    /// Optional file path to write the output results into
    #[arg(short, long, env, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Do NOT send the results to stdout
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_stdout: bool,

    /// Pretty format the result
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub pretty: bool,
}

/// A serializable/deserializable struct that bridges [QuoteResult] for cli
/// output by implementing `From` trait for it.
/// This is is needed since [crate::error::FailedQuote] does not impl ser/deser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "status", content = "message")]
pub enum QuoterResultInner {
    Error(String),
    #[serde(untagged)]
    Ok(OrderQuoteValue),
}

impl From<QuoteResult> for QuoterResultInner {
    fn from(value: QuoteResult) -> Self {
        match value {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Error(e.to_string()),
        }
    }
}

/// Wrapper struct for arrya of [QuoterResultInner]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct QuoterResult(pub Vec<QuoterResultInner>);

impl From<Vec<QuoteResult>> for QuoterResult {
    fn from(value: Vec<QuoteResult>) -> Self {
        Self(value.into_iter().map(|v| v.into()).collect())
    }
}

impl Quoter {
    /// Executes the CLI call based on the given options of self
    pub async fn run(&self) -> anyhow::Result<QuoterResult> {
        let result: QuoterResult = match self.input.read_content()? {
            InputContentType::Target(v) => v
                .do_quote(
                    self.rpc.as_str(),
                    self.block_number,
                    None,
                    self.multicall_address,
                )
                .await?
                .into(),
            InputContentType::Spec(v) => {
                if let Some(sg) = &self.subgraph {
                    v.do_quote(
                        sg.as_str(),
                        self.rpc.as_str(),
                        self.block_number,
                        None,
                        self.multicall_address,
                    )
                    .await?
                    .into()
                } else {
                    return Err(anyhow::anyhow!(
                        "requires '--subgraph' url to read orders details from"
                    ));
                }
            }
        };

        if !self.no_stdout || self.output.is_some() {
            let stringified_result = if self.pretty {
                serde_json::to_string_pretty::<QuoterResult>(&result)?
            } else {
                serde_json::to_string::<QuoterResult>(&result)?
            };
            if !self.no_stdout {
                let mut stdout = std::io::stdout().lock();
                stdout.write_all(stringified_result.as_bytes())?;
            }
            if let Some(v) = &self.output {
                write(v, stringified_result)?;
            }
        }

        Ok(result)
    }
}

/// The main entrypoint for this crate's cli
pub async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;
    let cli = Quoter::parse();
    cli.run().await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::FailedQuote, BatchQuoteSpec, QuoteSpec};
    use alloy::primitives::{hex::encode_prefixed, keccak256, B256, U256};
    use alloy::sol_types::{SolCall, SolValue};
    use alloy_ethers_typecast::{multicall::IMulticall3::Result as MulticallResult, rpc::Response};
    use clap::CommandFactory;
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::{quoteCall, OrderV3, IO};
    use std::{fs::read_to_string, str::FromStr};

    #[test]
    fn verify_cli() {
        Quoter::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let rpc = Url::parse("https://rpc.com").unwrap();
        let sg = Url::parse("https://sg.com").unwrap();
        let output = PathBuf::from_str("./a/b").unwrap();

        let batch_quote_specs = BatchQuoteSpec(vec![QuoteSpec {
            orderbook: Address::random(),
            input_io_index: 0,
            output_io_index: 0,
            order_hash: U256::from(1),
            signed_context: vec![],
        }]);
        let mut bytes = vec![];
        bytes.extend(batch_quote_specs.0[0].orderbook.0);
        bytes.push(batch_quote_specs.0[0].input_io_index);
        bytes.push(batch_quote_specs.0[0].output_io_index);
        bytes.extend(batch_quote_specs.0[0].order_hash.to_be_bytes_vec());
        let hex_bytes = encode_prefixed(&bytes);
        let cmd = Quoter::command();
        let result = cmd
            .try_get_matches_from(vec![
                "cmd",
                "--output",
                output.clone().to_str().unwrap(),
                "--rpc",
                rpc.as_str(),
                "-i",
                &hex_bytes,
                "--sg",
                sg.as_str(),
            ])
            .unwrap();
        assert_eq!(result.get_one::<PathBuf>("output"), Some(&output));
        assert_eq!(result.get_one::<Url>("subgraph"), Some(&sg));
        assert_eq!(result.get_one::<Url>("rpc"), Some(&rpc));
        assert_eq!(
            result.get_one::<BatchQuoteSpec>("input"),
            Some(&batch_quote_specs)
        );

        let orderbook1 = encode_prefixed(Address::random().0);
        let orderbook2 = encode_prefixed(Address::random().0);
        let order_bytes1 = encode_prefixed(OrderV3::default().abi_encode());
        let order_bytes2 = encode_prefixed(OrderV3::default().abi_encode());
        let input_index = U256::from(8).to_string();
        let output_index = U256::from(9).to_string();
        let cmd = Quoter::command();
        let result = cmd.get_matches_from(vec![
            "cmd",
            "--output",
            output.clone().to_str().unwrap(),
            "--rpc",
            rpc.as_str(),
            "--target",
            &orderbook1,
            &input_index,
            &output_index,
            &order_bytes1,
            "--target",
            &orderbook2,
            &input_index,
            &output_index,
            &order_bytes2,
        ]);
        assert_eq!(result.get_one::<PathBuf>("output"), Some(&output));
        assert_eq!(result.get_one::<Url>("rpc"), Some(&rpc));
        assert_eq!(
            result
                .get_occurrences("target")
                .unwrap()
                .map(Iterator::collect)
                .collect::<Vec<Vec<&String>>>(),
            vec![
                vec![&orderbook1, &input_index, &output_index, &order_bytes1],
                vec![&orderbook2, &input_index, &output_index, &order_bytes2]
            ]
        );

        let cmd = Quoter::command();
        assert!(cmd
            .try_get_matches_from(vec![
                "cmd",
                "--output",
                output.clone().to_str().unwrap(),
                "--target",
                &orderbook1,
                &input_index,
                &output_index,
                &order_bytes1,
                "--sg",
                sg.as_str(),
            ])
            .is_err());
    }

    #[tokio::test]
    async fn test_run_err() {
        let cli = Quoter {
            output: Some(PathBuf::new()),
            rpc: Url::parse("http://a.com").unwrap(),
            subgraph: None,
            block_number: None,
            multicall_address: None,
            no_stdout: true,
            pretty: true,
            input: Input {
                target: None,
                spec: None,
                input: Some(BatchQuoteSpec(vec![
                    QuoteSpec::default(),
                    QuoteSpec::default(),
                ])),
            },
        };
        let result = cli.run().await.expect_err("expected error").to_string();
        assert_eq!(
            result,
            "requires '--subgraph' url to read orders details from"
        );
    }

    #[tokio::test]
    async fn test_run_ok_spec_inputs() {
        let rpc_server = MockServer::start_async().await;
        let rpc_url = rpc_server.url("/rpc");
        let sg_url = rpc_server.url("/sg");

        let rpc_response_data = vec![
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(true, U256::ZERO, U256::ZERO)).into(),
            },
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(false, U256::ZERO, U256::ZERO)).into(),
            },
        ]
        .abi_encode();
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &serde_json::from_str::<serde_json::Value>(
                    &Response::new_success(1, encode_prefixed(rpc_response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        // mock subgraph
        let orderbook = Address::random();
        let order = OrderV3 {
            validInputs: vec![IO::default()],
            validOutputs: vec![IO::default()],
            ..Default::default()
        };
        let order_hash_bytes = keccak256(order.abi_encode()).0;
        let order_hash_u256 = U256::from_be_bytes(order_hash_bytes);
        let order_hash = encode_prefixed(order_hash_bytes);
        let mut order_id = vec![];
        order_id.extend_from_slice(orderbook.as_ref());
        order_id.extend_from_slice(&order_hash_bytes);
        let order_id = encode_prefixed(keccak256(order_id));
        let retrun_sg_data = serde_json::json!({
            "data": {
                "orders": [{
                    "id": order_id,
                    "orderBytes": encode_prefixed(order.abi_encode()),
                    "orderHash": order_hash,
                    "owner": encode_prefixed(order.owner),
                    "outputs": [{
                        "id": encode_prefixed(Address::random().0.0),
                        "owner": encode_prefixed(order.owner),
                        "token": {
                            "id": encode_prefixed(order.validOutputs[0].token.0.0),
                            "address": encode_prefixed(order.validOutputs[0].token.0.0),
                            "name": "T1",
                            "symbol": "T1",
                            "decimals": order.validOutputs[0].decimals.to_string()
                        },
                        "balance": "0",
                        "vaultId": order.validOutputs[0].vaultId.to_string(),
                        "orderbook": { "id": encode_prefixed(B256::random()) },
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    }],
                    "inputs": [{
                        "id": encode_prefixed(Address::random().0.0),
                        "owner": encode_prefixed(order.owner),
                        "token": {
                            "id": encode_prefixed(order.validInputs[0].token.0.0),
                            "address": encode_prefixed(order.validInputs[0].token.0.0),
                            "name": "T2",
                            "symbol": "T2",
                            "decimals": order.validInputs[0].decimals.to_string()
                        },
                        "balance": "0",
                        "vaultId": order.validInputs[0].vaultId.to_string(),
                        "orderbook": { "id": encode_prefixed(B256::random()) },
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    }],
                    "orderbook": { "id": encode_prefixed(B256::random()) },
                    "active": true,
                    "addEvents": [{
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                            "from": encode_prefixed(Address::random())
                        }
                    }],
                    "meta": null,
                    "timestampAdded": "0",
                    "trades": []
                }]
            }
        });
        rpc_server.mock(|when, then| {
            when.method(POST).path("/sg");
            then.json_body_obj(&retrun_sg_data);
        });

        // input bytes
        let batch_quote_specs = BatchQuoteSpec(vec![
            QuoteSpec {
                order_hash: order_hash_u256,
                input_io_index: 0,
                output_io_index: 0,
                signed_context: vec![],
                orderbook,
            },
            QuoteSpec::default(),
        ]);
        let cli = Quoter {
            output: None,
            rpc: Url::parse(&rpc_url).unwrap(),
            subgraph: Some(Url::parse(&sg_url).unwrap()),
            block_number: None,
            multicall_address: None,
            no_stdout: true,
            pretty: false,
            input: Input {
                target: None,
                spec: None,
                input: Some(batch_quote_specs),
            },
        };

        // run
        let result = cli.run().await.unwrap();
        let expected = QuoterResult(vec![
            QuoterResultInner::Ok(OrderQuoteValue::default()),
            QuoterResultInner::Error(FailedQuote::NonExistent.to_string()),
        ]);
        assert_eq!(result, expected);

        // specs input
        let specs_str = vec![
            encode_prefixed(orderbook.0),
            0.to_string(),
            0.to_string(),
            encode_prefixed(order_hash_bytes),
            encode_prefixed(orderbook.0),
            0.to_string(),
            0.to_string(),
            encode_prefixed([0u8; 32]),
        ];
        let cli = Quoter {
            output: None,
            rpc: Url::parse(&rpc_url).unwrap(),
            subgraph: Some(Url::parse(&sg_url).unwrap()),
            block_number: None,
            multicall_address: None,
            no_stdout: true,
            pretty: false,
            input: Input {
                target: None,
                input: None,
                spec: Some(specs_str),
            },
        };

        // run
        let result = cli.run().await.unwrap();
        let expected = QuoterResult(vec![
            QuoterResultInner::Ok(OrderQuoteValue::default()),
            QuoterResultInner::Error(FailedQuote::NonExistent.to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_run_ok_target_args() {
        let rpc_server = MockServer::start_async().await;
        let rpc_url = rpc_server.url("/rpc");
        let test_path = std::env::current_dir().unwrap().join("test-result.json");

        let orderbook = Address::random();
        let input_io_index = 0u8;
        let output_io_index = 0u8;
        let targets_str = vec![
            encode_prefixed(orderbook.0),
            input_io_index.to_string(),
            output_io_index.to_string(),
            encode_prefixed(OrderV3::default().abi_encode()),
        ];

        let cli = Quoter {
            output: Some(test_path.clone()),
            rpc: Url::parse(&rpc_url).unwrap(),
            subgraph: None,
            block_number: None,
            multicall_address: None,
            no_stdout: false,
            pretty: false,
            input: Input {
                input: None,
                spec: None,
                target: Some(targets_str),
            },
        };

        let rpc_response_data = vec![
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(true, U256::ZERO, U256::ZERO)).into(),
            },
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(false, U256::ZERO, U256::ZERO)).into(),
            },
        ]
        .abi_encode();
        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body_obj(
                &serde_json::from_str::<serde_json::Value>(
                    &Response::new_success(1, encode_prefixed(rpc_response_data).as_str())
                        .to_json_string()
                        .unwrap(),
                )
                .unwrap(),
            );
        });

        // run
        let result = cli.run().await.unwrap();
        let expected = QuoterResult(vec![
            QuoterResultInner::Ok(OrderQuoteValue::default()),
            QuoterResultInner::Error(FailedQuote::NonExistent.to_string()),
        ]);
        assert_eq!(result, expected);

        // output json format containing array of ok/err quote results:
        // [
        //     { "maxOutput": "0x0", "ratio": "0x0" },
        //     { "status": "error", "message": "Order does not exist" }
        // ]
        let result = read_to_string(test_path.clone()).unwrap();
        let expected = serde_json::to_string(&expected).unwrap();
        assert_eq!(result, expected);

        // rmeove the output test file
        std::fs::remove_file(test_path).unwrap();
    }
}
