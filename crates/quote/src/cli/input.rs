use crate::{BatchQuoteSpec, BatchQuoteTarget, QuoteSpec, QuoteTarget};
use alloy_primitives::{hex::FromHex, Address, U256};
use clap::Args;
use std::{fs::read_to_string, path::PathBuf};
use url::Url;

/// Determines the variants of parsed json input
#[derive(Debug, Clone, PartialEq)]
pub enum InputContentType {
    /// quote specs that need to read their order details from a subgraph before a quote call
    Spec(BatchQuoteSpec),
    // ready to quote targets that have all the details for a quote call
    Target(BatchQuoteTarget),
}

/// Group of valid input formats
/// Only one of them can be passed at a time in cli
#[derive(Args, Clone, Debug, PartialEq)]
#[group(required = true, multiple = false)]
pub struct Input {
    /// Input json file path
    #[arg(short, long, env, value_name = "PATH")]
    pub input: Option<PathBuf>,

    /// Remote input json url
    #[arg(long, env, value_name = "URL")]
    pub remote_json: Option<Url>,

    /// Input that is in json stringified format
    #[arg(short, long, env)]
    pub json_string: Option<String>,

    /// Input in simplest form that takes exactly 4 values (orderbook address,
    /// order hash, input index and output index) in order, "--subgraph-url" is
    /// required when usng this arg
    #[arg(
        short,
        long,
        num_args = 4,
        requires = "subgraph",
        value_names = ["ORDERBOOK_ADDRESS", "ORDER_HASH", "INPUT_IO_INDEX", "OUTPUT_IO_INDEX"],
    )]
    pub quote_spec: Option<Vec<String>>,
}

impl Input {
    /// Parses the given json string into one of the expected types
    pub fn resolve_type(json_string: &str) -> anyhow::Result<InputContentType> {
        if let Ok(v) = serde_json::from_str::<BatchQuoteTarget>(json_string) {
            return Ok(InputContentType::Target(v));
        }
        if let Ok(v) = serde_json::from_str::<QuoteTarget>(json_string) {
            return Ok(InputContentType::Target(BatchQuoteTarget(vec![v])));
        }
        if let Ok(v) = serde_json::from_str::<BatchQuoteSpec>(json_string) {
            return Ok(InputContentType::Spec(v));
        }
        if let Ok(v) = serde_json::from_str::<QuoteSpec>(json_string) {
            return Ok(InputContentType::Spec(BatchQuoteSpec(vec![v])));
        }
        Err(anyhow::anyhow!("invalid json content"))
    }

    /// Reads the input content from the provided source
    pub async fn read_content(&self) -> anyhow::Result<InputContentType> {
        if let Some(path) = &self.input {
            Ok(Self::resolve_type(&read_to_string(path)?)?)
        } else if let Some(url) = &self.remote_json {
            Ok(Self::resolve_type(
                &reqwest::get(url.as_str()).await?.text().await?,
            )?)
        } else if let Some(json_string) = &self.json_string {
            Ok(Self::resolve_type(json_string)?)
        } else if let Some(specs) = &self.quote_spec {
            let mut batch_quote_specs = BatchQuoteSpec::default();
            let mut iter = specs.iter();
            while let Some(orderbook) = iter.next() {
                if let Some(order_hash_str) = iter.next() {
                    if let Some(input_io_index) = iter.next() {
                        if let Some(output_io_index) = iter.next() {
                            batch_quote_specs.0.push(QuoteSpec {
                                orderbook: Address::from_hex(orderbook)?,
                                order_hash: U256::from_str_radix(
                                    order_hash_str.strip_prefix("0x").unwrap_or(order_hash_str),
                                    16,
                                )?,
                                input_io_index: if input_io_index.starts_with("0x") {
                                    U256::from_str_radix(
                                        input_io_index.strip_prefix("0x").unwrap(),
                                        16,
                                    )?
                                } else {
                                    U256::from_str_radix(input_io_index, 10)?
                                },
                                output_io_index: if output_io_index.starts_with("0x") {
                                    U256::from_str_radix(
                                        output_io_index.strip_prefix("0x").unwrap(),
                                        16,
                                    )?
                                } else {
                                    U256::from_str_radix(output_io_index, 10)?
                                },
                                signed_context: vec![],
                            });
                        } else {
                            return Err(anyhow::anyhow!("missing output IO index"));
                        }
                    } else {
                        return Err(anyhow::anyhow!("missing input IO index"));
                    }
                } else {
                    return Err(anyhow::anyhow!("missing order hash"));
                }
            }
            if batch_quote_specs.0.is_empty() {
                return Err(anyhow::anyhow!("missing '--quote-spec' values"));
            }
            Ok(InputContentType::Spec(batch_quote_specs))
        } else {
            Err(anyhow::anyhow!("unexpected input"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use alloy_primitives::hex::encode_prefixed;
    use httpmock::{Method::GET, MockServer};

    #[test]
    fn test_resolve_type() {
        let target = QuoteTarget::default();
        let result = Input::resolve_type(&serde_json::to_string(&target).unwrap()).unwrap();
        let expected = InputContentType::Target(BatchQuoteTarget(vec![target]));
        assert_eq!(result, expected);

        let target = BatchQuoteTarget(vec![QuoteTarget::default()]);
        let result = Input::resolve_type(&serde_json::to_string(&target).unwrap()).unwrap();
        let expected = InputContentType::Target(target);
        assert_eq!(result, expected);

        let spec = QuoteSpec::default();
        let result = Input::resolve_type(&serde_json::to_string(&spec).unwrap()).unwrap();
        let expected = InputContentType::Spec(BatchQuoteSpec(vec![spec]));
        assert_eq!(result, expected);

        let spec = BatchQuoteSpec(vec![QuoteSpec::default()]);
        let result = Input::resolve_type(&serde_json::to_string(&spec).unwrap()).unwrap();
        let expected = InputContentType::Spec(spec);
        assert_eq!(result, expected);

        let some_json = serde_json::to_string(&serde_json::json!({
            "someKey": 123,
        }))
        .unwrap();
        let result = Input::resolve_type(&some_json)
            .expect_err("expected error")
            .to_string();
        assert_eq!(result, "invalid json content");
    }

    #[tokio::test]
    async fn test_read_content() {
        // valid spec
        let orderbook1 = Address::random();
        let orderbook2 = Address::random();
        let order_hash1 = U256::from(1);
        let order_hash2 = U256::from(2);
        let input_index = U256::from(8);
        let output_index = U256::from(9);
        let input = Input {
            input: None,
            remote_json: None,
            json_string: None,
            quote_spec: Some(vec![
                encode_prefixed(orderbook1.0),
                encode_prefixed(order_hash1.to_be_bytes_vec()),
                input_index.to_string(),
                output_index.to_string(),
                encode_prefixed(orderbook2.0),
                encode_prefixed(order_hash2.to_be_bytes_vec()),
                input_index.to_string(),
                output_index.to_string(),
            ]),
        };
        let result = input.read_content().await.unwrap();
        let expected = InputContentType::Spec(BatchQuoteSpec(vec![
            QuoteSpec {
                order_hash: order_hash1,
                orderbook: orderbook1,
                input_io_index: input_index,
                output_io_index: output_index,
                signed_context: vec![],
            },
            QuoteSpec {
                order_hash: order_hash2,
                orderbook: orderbook2,
                input_io_index: input_index,
                output_io_index: output_index,
                signed_context: vec![],
            },
        ]));
        assert_eq!(result, expected);

        // missing spec value
        let input = Input {
            input: None,
            remote_json: None,
            json_string: None,
            quote_spec: Some(vec![
                encode_prefixed(orderbook1.0),
                encode_prefixed(order_hash1.to_be_bytes_vec()),
                input_index.to_string(),
            ]),
        };
        let result = input
            .read_content()
            .await
            .expect_err("expected error")
            .to_string();
        assert_eq!(result, "missing output IO index");

        // inline json string input
        let targets = BatchQuoteTarget(vec![QuoteTarget::default(), QuoteTarget::default()]);
        let input = Input {
            input: None,
            remote_json: None,
            quote_spec: None,
            json_string: Some(serde_json::to_string(&targets).unwrap()),
        };
        let result = input.read_content().await.unwrap();
        let expected = InputContentType::Target(targets);
        assert_eq!(result, expected);

        // file input
        let targets = BatchQuoteTarget(vec![QuoteTarget::default(), QuoteTarget::default()]);
        let contents = serde_json::to_string(&targets).unwrap();
        let test_path = std::env::current_dir().unwrap().join("test.json");
        fs::write(test_path.clone(), contents).unwrap();
        let input = Input {
            input: Some(test_path.clone()),
            remote_json: None,
            quote_spec: None,
            json_string: None,
        };
        let result = input.read_content().await.unwrap();
        let expected = InputContentType::Target(targets);
        assert_eq!(result, expected);
        fs::remove_file(test_path).unwrap();

        // remote input
        let rpc_server = MockServer::start_async().await;
        let targets = BatchQuoteTarget(vec![QuoteTarget::default(), QuoteTarget::default()]);
        rpc_server.mock(|when, then| {
            when.method(GET).path("/remote");
            then.json_body_obj(&targets);
        });
        let input = Input {
            input: None,
            remote_json: Some(url::Url::parse(&rpc_server.url("/remote")).unwrap()),
            quote_spec: None,
            json_string: None,
        };
        let result = input.read_content().await.unwrap();
        let expected = InputContentType::Target(targets);
        assert_eq!(result, expected);
    }
}
