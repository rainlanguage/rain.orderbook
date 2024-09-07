use crate::QuoteTarget;

use alloy::primitives::Address;
use rain_interpreter_eval::{
    error::ForkCallError,
    fork::{Forker, NewForkedEvm},
    trace::RainEvalResult,
};
use rain_orderbook_bindings::IOrderBookV4::quoteCall;
use url::Url;

pub struct NewQuoteDebugger {
    pub fork_url: Url,
}
pub struct QuoteDebugger {
    forker: Forker,
}

#[derive(Debug, thiserror::Error)]
pub enum QuoteDebuggerError {
    #[error("Forker error: {0}")]
    ForkerError(#[from] ForkCallError),
    #[error("Quote error: {0}")]
    QuoteError(#[from] crate::error::Error),
}

impl QuoteDebugger {
    pub async fn new(args: NewQuoteDebugger) -> Result<Self, QuoteDebuggerError> {
        let forker = Forker::new_with_fork(
            NewForkedEvm {
                fork_url: args.fork_url.to_string(),
                fork_block_number: None,
            },
            None,
            None,
        )
        .await?;

        Ok(Self { forker })
    }

    pub async fn debug(
        &mut self,
        quote_target: QuoteTarget,
    ) -> Result<RainEvalResult, QuoteDebuggerError> {
        quote_target.validate()?;

        let quote_call = quoteCall {
            quoteConfig: quote_target.quote_config.clone(),
        };

        let res = self
            .forker
            .alloy_call(Address::default(), quote_target.orderbook, quote_call, true)
            .await?;

        Ok(res.raw.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::network::TransactionBuilder;
    use alloy::primitives::utils::parse_ether;
    use alloy::primitives::U256;
    use alloy::rpc::types::TransactionRequest;
    use alloy::sol_types::{SolCall, SolValue};
    use rain_orderbook_bindings::IOrderBookV4::{OrderV3, Quote};
    use rain_orderbook_common::add_order::AddOrderArgs;
    use rain_orderbook_common::dotrain_order::DotrainOrder;
    use rain_orderbook_test_fixtures::{ContractTxHandler, LocalEvm};
    use std::str::FromStr;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_quote_debugger() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;
        let token1_holder = local_evm.signer_wallets[0].default_signer().address();
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let dotrain = format!(
            r#"
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
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
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

        let order = DotrainOrder::new(dotrain.clone(), None).await.unwrap();
        let deployment = order.config().deployments["some-key"].as_ref().clone();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
            .await
            .unwrap()
            .try_into_call(local_evm.url())
            .await
            .unwrap()
            .abi_encode();
        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address())
            .with_from(token1_holder);
        local_evm.send_transaction(tx).await.unwrap();
        let filter = orderbook.AddOrderV2_filter();
        let logs = filter.query().await.unwrap();
        let order = logs[0].0.order.clone();

        // approve and deposit Token1
        token1
            .approve(*orderbook.address(), parse_ether("1000").unwrap())
            .do_send(&local_evm.provider)
            .await
            .unwrap();
        orderbook
            .deposit2(
                *token1.address(),
                U256::from(0x01),
                parse_ether("1000").unwrap(),
                vec![],
            )
            .do_send(&local_evm.provider)
            .await
            .unwrap();

        let mut debugger = QuoteDebugger::new(NewQuoteDebugger {
            fork_url: Url::from_str(&local_evm.url()).unwrap(),
        })
        .await
        .unwrap();

        let order = OrderV3::abi_decode(&order.abi_encode(), true).unwrap();

        let quote_target = QuoteTarget {
            orderbook: *orderbook.address(),
            quote_config: Quote {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            },
        };

        let res = debugger.debug(quote_target).await.unwrap();

        assert_eq!(res.traces.len(), 1);
        assert_eq!(
            res.traces[0].stack,
            vec![parse_ether("52").unwrap(), parse_ether("16").unwrap()]
        );
    }
}
