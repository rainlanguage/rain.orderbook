use crate::{OrderQuoteValue, QuoteResult};
use alloy_primitives::Address;
use clap::{command, ArgAction, Parser};
use serde::{Deserialize, Serialize};
use std::{fs::write, path::PathBuf};
use url::Url;

mod input;
pub use input::*;

/// CLI app entrypoint sruct
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about = "Rain Orderbook Qoute CLI", long_about = None)]
pub struct QuoterCLi {
    #[command(flatten)]
    pub input: Input,

    /// Specifies the output file path
    #[arg(short, long, env, value_name = "PATH")]
    pub output: PathBuf,

    /// RPC URL of the evm chain to quote
    #[arg(short, long, env, value_name = "URL", hide_env_values = true)]
    pub rpc: Url,

    /// Subgraph URL to read orders details from, presence of this
    /// arg determines what type input's undelying content should be in
    #[arg(short, long, env, value_name = "URL", visible_alias = "sg")]
    pub subgraph: Option<Url>,

    /// Optional block number to quote at
    #[arg(short, long, env, value_name = "INTEGER")]
    pub block_number: Option<u64>,

    /// Optional multicall3 address to use when quoting
    #[arg(short, long, env, value_name = "ADDRESS")]
    pub multicall_address: Option<Address>,

    /// Log the results
    #[arg(long, action = ArgAction::SetTrue)]
    pub stdout: bool,

    /// Pretty format the result
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub pretty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "status", content = "message")]
pub enum QuoteCliResultInner {
    Error(String),
    #[serde(untagged)]
    Ok(OrderQuoteValue),
}

impl From<QuoteResult> for QuoteCliResultInner {
    fn from(value: QuoteResult) -> Self {
        match value {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Error(e.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QuoteCliResult(pub Vec<QuoteCliResultInner>);

impl From<Vec<QuoteResult>> for QuoteCliResult {
    fn from(value: Vec<QuoteResult>) -> Self {
        Self(value.into_iter().map(|v| v.into()).collect())
    }
}

/// Dispatches the CLI call based on the given options
pub async fn dispatch(cli: QuoterCLi) -> anyhow::Result<()> {
    let strigifier = if cli.pretty {
        serde_json::to_string_pretty::<QuoteCliResult>
    } else {
        serde_json::to_string::<QuoteCliResult>
    };
    let result = match cli.input.read_content().await? {
        InputContentType::Target(v) => strigifier(
            &v.do_quote(cli.rpc.as_str(), cli.block_number, cli.multicall_address)
                .await?
                .into(),
        )?,
        InputContentType::Spec(v) => {
            if let Some(sg) = cli.subgraph {
                strigifier(
                    &v.do_quote(
                        sg.as_str(),
                        cli.rpc.as_str(),
                        cli.block_number,
                        cli.multicall_address,
                    )
                    .await?
                    .into(),
                )?
            } else {
                return Err(anyhow::anyhow!(
                    "requires '--subgraph' url to read orders details from"
                ));
            }
        }
    };
    if cli.stdout {
        println!("{}", result);
    }
    write(cli.output, result)?;

    Ok(())
}

/// The main entrypoint for this crate's cli
pub async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;
    let cli = QuoterCLi::parse();
    dispatch(cli).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::FailedQuote, BatchQuoteSpec, BatchQuoteTarget, QuoteSpec, QuoteTarget};
    use alloy_ethers_typecast::{multicall::IMulticall3::Result as MulticallResult, rpc::Response};
    use alloy_primitives::{hex::encode_prefixed, U256};
    use alloy_sol_types::{SolCall, SolValue};
    use clap::CommandFactory;
    use httpmock::{Method::POST, MockServer};
    use rain_orderbook_bindings::IOrderBookV4::quoteCall;
    use std::{fs::read_to_string, str::FromStr};

    #[test]
    fn verify_cli() {
        QuoterCLi::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let orderbook1 = encode_prefixed(Address::random().0);
        let orderbook2 = encode_prefixed(Address::random().0);
        let order_hash1 = encode_prefixed(U256::from(1).to_be_bytes_vec());
        let order_hash2 = encode_prefixed(U256::from(2).to_be_bytes_vec());
        let input_index = U256::from(8).to_string();
        let output_index = U256::from(9).to_string();
        let rpc = Url::parse("https://rpc.com").unwrap();
        let sg = Url::parse("https://sg.com").unwrap();
        let output = PathBuf::from_str("./a/b").unwrap();

        let cmd = QuoterCLi::command();
        let result = cmd.get_matches_from(vec![
            "cmd",
            "--output",
            output.clone().to_str().unwrap(),
            "--rpc",
            rpc.as_str(),
            "--quote-spec",
            &orderbook1,
            &order_hash1,
            &input_index,
            &output_index,
            "--quote-spec",
            &orderbook2,
            &order_hash2,
            &input_index,
            &output_index,
            "--sg",
            sg.as_str(),
        ]);

        assert_eq!(result.get_one::<PathBuf>("output"), Some(&output));
        assert_eq!(result.get_one::<Url>("subgraph"), Some(&sg));
        assert_eq!(result.get_one::<Url>("rpc"), Some(&rpc));
        assert_eq!(
            result
                .get_occurrences("quote_spec")
                .unwrap()
                .map(Iterator::collect)
                .collect::<Vec<Vec<&String>>>(),
            vec![
                vec![&orderbook1, &order_hash1, &input_index, &output_index,],
                vec![&orderbook2, &order_hash2, &input_index, &output_index,]
            ]
        );

        let cmd = QuoterCLi::command();
        assert!(cmd
            .try_get_matches_from(vec![
                "cmd",
                "--output",
                output.clone().to_str().unwrap(),
                "--rpc",
                rpc.as_str(),
                "--quote-spec",
                &orderbook1,
                &order_hash1,
                &input_index,
                &output_index,
            ])
            .is_err());

        let cmd = QuoterCLi::command();
        assert!(cmd
            .try_get_matches_from(vec![
                "cmd",
                "--rpc",
                rpc.as_str(),
                "--quote-spec",
                &orderbook1,
                &order_hash1,
                &input_index,
                &output_index,
                "--sg",
                sg.as_str(),
            ])
            .is_err());

        let cmd = QuoterCLi::command();
        assert!(cmd
            .try_get_matches_from(vec![
                "cmd",
                "--output",
                output.clone().to_str().unwrap(),
                "--quote-spec",
                &orderbook1,
                &order_hash1,
                &input_index,
                &output_index,
                "--sg",
                sg.as_str(),
            ])
            .is_err());
    }

    #[tokio::test]
    async fn test_dispatch_err() {
        let cli = QuoterCLi {
            output: PathBuf::new(),
            rpc: Url::parse("http://a.com").unwrap(),
            subgraph: None,
            block_number: None,
            multicall_address: None,
            stdout: true,
            pretty: true,
            input: Input {
                input: None,
                remote_json: None,
                quote_spec: None,
                json_string: Some(
                    serde_json::to_string(&BatchQuoteSpec(vec![QuoteSpec::default()])).unwrap(),
                ),
            },
        };
        let result = dispatch(cli).await.expect_err("expected error").to_string();
        assert_eq!(
            result,
            "requires '--subgraph' url to read orders details from"
        );
    }

    #[tokio::test]
    async fn test_dispatch_ok() {
        let rpc_server = MockServer::start_async().await;
        let rpc_url = rpc_server.url("/rpc");
        let test_path = std::env::current_dir().unwrap().join("test-result.json");

        let targets = BatchQuoteTarget(vec![QuoteTarget::default()]);
        let cli = QuoterCLi {
            output: test_path.clone(),
            rpc: Url::parse(&rpc_url).unwrap(),
            subgraph: None,
            block_number: None,
            multicall_address: None,
            stdout: true,
            pretty: false,
            input: Input {
                input: None,
                remote_json: None,
                quote_spec: None,
                json_string: Some(serde_json::to_string(&targets).unwrap()),
            },
        };

        let rpc_response_data = vec![
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(true, U256::ZERO, U256::ZERO)),
            },
            MulticallResult {
                success: true,
                returnData: quoteCall::abi_encode_returns(&(false, U256::ZERO, U256::ZERO)),
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

        // run dispatch
        dispatch(cli).await.unwrap();

        // output json format containing ok and err variants:
        // [
        //     { "maxOutput": "0x0", "ratio": "0x0" },
        //     { "status": "error", "message": "Order does not exist" }
        // ]
        let result = read_to_string(test_path.clone()).unwrap();
        let expected = serde_json::to_string(&QuoteCliResult(vec![
            QuoteCliResultInner::Ok(OrderQuoteValue::default()),
            QuoteCliResultInner::Error(FailedQuote::NonExistent.to_string()),
        ]))
        .unwrap();
        assert_eq!(result, expected);

        // rmeove the output test file
        std::fs::remove_file(test_path).unwrap();
    }
}
