use crate::error::{CommandError, CommandResult};
use alloy::primitives::{Address, U256};
use rain_orderbook_bindings::IOrderBookV4::Quote;
use rain_orderbook_common::fuzz::{RainEvalResults, RainEvalResultsTable};
use rain_orderbook_quote::{NewQuoteDebugger, QuoteDebugger, QuoteTarget};
use rain_orderbook_subgraph_client::types::common::*;

#[tauri::command]
pub async fn debug_order_quote(
    order: SgOrder,
    input_io_index: u32,
    output_io_index: u32,
    orderbook: Address,
    rpcs: Vec<String>,
    block_number: Option<u32>,
) -> CommandResult<(RainEvalResultsTable, Option<String>)> {
    let quote_target = QuoteTarget {
        orderbook,
        quote_config: Quote {
            order: order.try_into()?,
            inputIOIndex: U256::from(input_io_index),
            outputIOIndex: U256::from(output_io_index),
            signedContext: vec![],
        },
    };

    let mut debugger: Option<QuoteDebugger> = None;
    let mut err = None;
    for rpc in rpcs {
        match QuoteDebugger::new(NewQuoteDebugger {
            fork_url: rpc.parse()?,
            fork_block_number: block_number.map(|s| s.into()),
        })
        .await
        {
            Ok(res) => {
                debugger = Some(res);
            }
            Err(e) => {
                err = Some(CommandError::QuoteDebuggerError(e));
            }
        }
    }
    if let Some(err) = err {
        return Err(err);
    }
    // debugger should be some here
    let mut debugger = debugger.unwrap();

    let res = debugger.debug(quote_target).await?;
    let eval_res: RainEvalResults = vec![res.0.clone()].into();

    Ok((
        eval_res.into_flattened_table()?,
        res.1.map(|v| match v {
            Ok(e) => e.to_string(),
            Err(e) => e.to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::utils::parse_ether,
        sol_types::{SolCall, SolValue},
    };
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_test_fixtures::LocalEvm;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_debug_order_quote() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let dotrain = format!(
            r#"
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
            key: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key !Test binding
#calculate-io
amount price: 16 52;
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

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.clone(), None)
            .await
            .unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(vec![local_evm.url()])
            .await
            .unwrap()
            .abi_encode();

        let order = local_evm
            .add_order_and_deposit(
                &calldata,
                token1_holder,
                *token1.address(),
                parse_ether("1000").unwrap(),
                U256::from(1),
            )
            .await
            .0
            .order;

        let order = SgOrder {
            id: SgBytes("0x01".to_string()),
            orderbook: SgOrderbook {
                id: SgBytes(orderbook.address().to_string()),
            },
            order_bytes: SgBytes(encode_prefixed(order.abi_encode())),
            order_hash: SgBytes("0x01".to_string()),
            owner: SgBytes("0x01".to_string()),
            outputs: vec![],
            inputs: vec![],
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: SgBigInt(0.to_string()),
            trades: vec![],
            remove_events: vec![],
        };

        let input_io_index = 0;
        let output_io_index = 0;

        let rpc_url = local_evm.url();

        let result = debug_order_quote(
            order,
            input_io_index,
            output_io_index,
            *orderbook.address(),
            vec![rpc_url],
            None,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().0.rows[0],
            [parse_ether("16").unwrap(), parse_ether("52").unwrap()]
        );
    }
}
