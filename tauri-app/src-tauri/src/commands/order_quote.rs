use crate::error::CommandResult;
use alloy::primitives::{Address, U256};
use rain_orderbook_bindings::IOrderBookV4::Quote;
use rain_orderbook_common::fuzz::{RainEvalResults, RainEvalResultsTable};
use rain_orderbook_quote::{
    BatchQuoteTarget, NewQuoteDebugger, OrderQuoteValue, QuoteDebugger, QuoteTarget,
};
use rain_orderbook_subgraph_client::types::order_detail;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct BatchOrderQuotesResponse {
    pub pair: Pair,
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    pub error: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Pair {
    pub pair_name: String,
    pub input_index: u32,
    pub output_index: u32,
}

#[tauri::command]
pub async fn batch_order_quotes(
    orders: Vec<order_detail::Order>,
    rpc_url: String,
) -> CommandResult<Vec<BatchOrderQuotesResponse>> {
    let mut results: Vec<BatchOrderQuotesResponse> = Vec::new();

    for order in &orders {
        let mut pairs: Vec<Pair> = Vec::new();
        let mut quote_targets: Vec<QuoteTarget> = Vec::new();
        let orderbook = Address::from_str(&order.orderbook.id.0)?;

        for (input_index, input) in order.inputs.iter().enumerate() {
            for (output_index, output) in order.outputs.iter().enumerate() {
                let pair_name = format!(
                    "{}/{}",
                    output.token.symbol.as_deref().unwrap_or("UNKNOWN"),
                    input.token.symbol.as_deref().unwrap_or("UNKNOWN"),
                );

                let quote = order.clone().try_into()?;
                let quote_target = QuoteTarget {
                    orderbook,
                    quote_config: Quote {
                        order: quote,
                        inputIOIndex: U256::from(input_index),
                        outputIOIndex: U256::from(output_index),
                        signedContext: vec![],
                    },
                };

                if input_index != output_index {
                    pairs.push(Pair {
                        pair_name,
                        input_index: input_index as u32,
                        output_index: output_index as u32,
                    });
                    quote_targets.push(quote_target);
                }
            }
        }

        let quote_values = BatchQuoteTarget(quote_targets)
            .do_quote(&rpc_url, None, None)
            .await;

        if let Ok(quote_values) = quote_values {
            for (quote_value_result, pair) in quote_values.into_iter().zip(pairs) {
                match quote_value_result {
                    Ok(quote_value) => {
                        results.push(BatchOrderQuotesResponse {
                            pair,
                            success: true,
                            data: Some(quote_value),
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(BatchOrderQuotesResponse {
                            pair,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        } else if let Err(e) = quote_values {
            for pair in pairs {
                results.push(BatchOrderQuotesResponse {
                    pair,
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn debug_order_quote(
    order: order_detail::Order,
    inputIOIndex: u32,
    outputIOIndex: u32,
    orderbook: Address,
    rpc_url: String,
) -> CommandResult<RainEvalResultsTable> {
    let quote_target = QuoteTarget {
        orderbook,
        quote_config: Quote {
            order: order.try_into()?,
            inputIOIndex: U256::from(inputIOIndex),
            outputIOIndex: U256::from(outputIOIndex),
            signedContext: vec![],
        },
    };

    let mut debugger = QuoteDebugger::new(NewQuoteDebugger {
        fork_url: rpc_url.parse()?,
    })
    .await?;

    let res: RainEvalResults = vec![debugger.debug(quote_target).await?].into();

    Ok(res.into_flattened_table()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CommandError;
    use rain_orderbook_common::subgraph::SubgraphArgs;
    use rain_orderbook_env::CI_DEPLOY_POLYGON_RPC_URL;
    use rain_orderbook_subgraph_client::types::order_detail::{self, Bytes};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_debug_order_quote() {
        let order = order_detail::Order {
            id: order_detail::RainMetaV1::default(),
            order_bytes: Bytes("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000008e4bdeec7ceb9570d440676345da1dce10329f5b00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000076000000000000000000000000000000000000000000000000000000000000007e0eccd7aa9a9f8af012d11a874253fdd8b48fd35c63f21cf97d0c33f6d141268db0000000000000000000000006352593f4018c99df731de789e2a147c7fb29370000000000000000000000000de38ad4b13d5258a5653e530ecdf0ca71b4e8a51000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000006250000000000000000000000000000000000000000000000000000000000000019000000000000000000000000000000000000000000000001158e460913d000000000000000000000000000000000000000000000000000001bc16d674ec8000000000000000000000000000000000000000000000000000340aad21b3b7000008d5061727469616c2074726164650000000000000000000000000000000000008c636f6f6c646f776e2d6b65790000000000000000000000000000000000000088636f6f6c646f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de0b6b3a764000000000000000000000000000000000000000000000000000029a2241af62c00000000000000000000000000000000000000000000000000003782dace9d90000000000000000000000000000000000000000000000000000003782dace9d900008764656661756c7400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b20000000000000000000000000000e1b3eb06806601828976e491914e3de18b5d6b280000000000000000000000003c499c542cef5e3811e1192ce70d8cc03d5c33590000000000000000000000005757371414417b8c6caad45baef941abc7d3ab3296e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f00000000000000000000000000000000000000000000000000b1a2bc2ec50000896d696e20726174696f000000000000000000000000000000000000000000008d5072696365206368616e67652e000000000000000000000000000000000000000000000000000000000000000000000000000000000000002386f26fc1000000000000000000000000000000000000000000000000000000470de4df8200000000000000000000000000000000000000000000000000008ac7230489e800000000000000000000000000000000000000000000000000000214e8348c4f0000000000000000000000010001bc609623f5020f6fc7481024862cd5ee3fff52d700000000000000000000000000000000000000000000000000000000000002c50c00000060008000c801b401e00204021c0220026c02840288170900070b200002001000000b1100030010000201100001011000003d1200003d120000001000030b11000401100002001000012b120000001000004812000100100004001000030b230005001000060b01000600100006001000050b020007070300000110000303100002031001044412000003100404211200001d020000110500021a10000001100004031000010c120000491100000110000501100002001000012b12000000100000211200001d0200000010000001100004031000010c1200004a0200003a14010701100006001000000b12000801100007001000000b12000801100001001000000b12000801100008001000000b12000801100009001000000b1200080110000c0110000b001000050110000700100005251200000110000a00100005211200001f120000001000040110000700100004251200000110000a00100004211200001f120000001000030110000700100003251200000110000a00100003211200001f120000001000020110000700100002251200000110000a00100002211200001f120000001000010110000700100001251200000110000a00100001211200001f1200001c1c00000a060104011000100110000f001000000110000e0110000d02250018001000020b010009001000010b11000a0806030600100000001000020b11000b00100001481200012e120000001000000010000305040101011000120110001100100000201200001d020000000202021207020600100001001000000c12000007111400061100000110000701100006001000000c1200003c12000000100003001000022b12000001100000011000072b120000001000042e12000005040101011000131a10000000100000241200001d0200000001010108050102011000170010000001100016011000152e12000001100014271300003b12000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000003c499c542cef5e3811e1192ce70d8cc03d5c33590000000000000000000000000000000000000000000000000000000000000006d302fedcbf6b3ea84812cde736439a97478b93fce4b546bc445f837f255893840000000000000000000000000000000000000000000000000000000000000001000000000000000000000000e1b3eb06806601828976e491914e3de18b5d6b280000000000000000000000000000000000000000000000000000000000000012d302fedcbf6b3ea84812cde736439a97478b93fce4b546bc445f837f25589384".to_string()) ,
            order_hash: Bytes("0x01".to_string()),
            owner: Bytes("0x01".to_string()),
            outputs: vec![],
            inputs: vec![],
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: 0.into(),
        };

        let inputIOIndex = 0;
        let outputIOIndex = 0;

        let orderbook = Address::from_str("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb").unwrap();

        let rpc_url = CI_DEPLOY_POLYGON_RPC_URL.to_string();

        let result =
            debug_order_quote(order, inputIOIndex, outputIOIndex, orderbook, rpc_url).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows[0].len(), 8);
    }
}
