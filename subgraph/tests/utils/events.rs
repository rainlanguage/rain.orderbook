use crate::generated::{
    AddOrderFilter, AfterClearFilter, ClearFilter, ContextFilter, DepositFilter, OrderBook,
    TakeOrderFilter, WithdrawFilter,
};
use crate::generated::{ERC20Mock, TransferFilter};
use crate::generated::{NewExpressionFilter, RainterpreterExpressionDeployer};
use anyhow::{Error, Result};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, PendingTransaction, Provider},
    signers::Wallet,
    types::{Filter, Log, Topic, TransactionReceipt, TxHash},
};

use super::get_provider;

/// Try to extract the hash value from a Topic (ValueOrArray) type
// fn extract_topic_hash(topic: ValueOrArray<Option<TxHash>>) -> Option<TxHash> {
fn extract_topic_hash(filter: Filter) -> Option<TxHash> {
    let option_topic = filter.topics[0].clone();

    if let Some(topic) = option_topic {
        match topic {
            Topic::Value(Some(data)) => return Some(data),
            Topic::Array(topic) => {
                if let Some(data) = topic.get(0) {
                    return data.clone();
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }

    None
}

/// Get the first log in a transaction that match the filter
async fn get_matched_log(receipt: TransactionReceipt, filter: Filter) -> Option<Log> {
    let topic_hash = extract_topic_hash(filter);

    if let Some(hash) = topic_hash {
        for log in receipt.logs {
            if let Some(first_topic) = log.topics.get(0) {
                if first_topic == &hash {
                    return Some(log.clone());
                }
            }
        }
    }

    None
}

/// Get all the logs in a transaction for a given matched filter.
async fn get_matched_logs(receipt: TransactionReceipt, filter: Filter) -> Option<Vec<Log>> {
    let topic_hash = extract_topic_hash(filter);

    if let Some(hash) = topic_hash {
        let mut logs: Vec<Log> = Vec::new();
        //
        for log in receipt.logs {
            if let Some(first_topic) = log.topics.get(0) {
                if first_topic == &hash {
                    logs.push(log.clone())
                }
            }
        }

        if logs.len() > 0 {
            return Some(logs);
        }
    }

    None
}

async fn get_pending_tx(tx_hash: &TxHash) -> Result<TransactionReceipt> {
    let provider = get_provider().await?;

    let pending_tx = PendingTransaction::new(*tx_hash, provider);

    match pending_tx.await? {
        Some(receipt) => return Ok(receipt),
        None => return Err(Error::msg("receipt not found")),
    };
}

pub async fn get_add_order_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<AddOrderFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().add_order_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event =
                contract.decode_event::<AddOrderFilter>("AddOrder", log.topics, log.data)?;

            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn _get_add_order_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<AddOrderFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().add_order_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<AddOrderFilter> = Vec::new();

            for log in logs {
                let event: AddOrderFilter =
                    contract.decode_event::<AddOrderFilter>("AddOrder", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn _get_clear_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<ClearFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clear_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event = contract.decode_event::<ClearFilter>("Clear", log.topics, log.data)?;
            return Ok(event);
        }
        None => return Err(Error::msg("receipt not found")),
    }
}

pub async fn _get_after_clear_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<AfterClearFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.after_clear_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event =
                contract.decode_event::<AfterClearFilter>("AfterClear", log.topics, log.data)?;
            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn _get_transfer_event(
    contract: ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<TransferFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter = contract.transfer_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event =
                contract.decode_event::<TransferFilter>("Transfer", log.topics, log.data)?;
            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn _get_new_expression_event(
    contract: RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<NewExpressionFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().new_expression_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event = contract.decode_event::<NewExpressionFilter>(
                "NewExpression",
                log.topics,
                log.data,
            )?;
            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn get_withdraw_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<WithdrawFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().withdraw_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<WithdrawFilter> = Vec::new();

            for log in logs {
                let event: WithdrawFilter =
                    contract.decode_event::<WithdrawFilter>("Withdraw", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn get_deposit_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<DepositFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().deposit_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<DepositFilter> = Vec::new();

            for log in logs {
                let event: DepositFilter =
                    contract.decode_event::<DepositFilter>("Deposit", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn get_clear_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<ClearFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().clear_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<ClearFilter> = Vec::new();

            for log in logs {
                let event: ClearFilter =
                    contract.decode_event::<ClearFilter>("Clear", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn get_after_clear_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<AfterClearFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().after_clear_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<AfterClearFilter> = Vec::new();

            for log in logs {
                let event: AfterClearFilter = contract.decode_event::<AfterClearFilter>(
                    "AfterClear",
                    log.topics,
                    log.data,
                )?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn get_take_order_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<TakeOrderFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.take_order_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event =
                contract.decode_event::<TakeOrderFilter>("TakeOrder", log.topics, log.data)?;
            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn get_take_order_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<TakeOrderFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().take_order_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<TakeOrderFilter> = Vec::new();

            for log in logs {
                let event: TakeOrderFilter =
                    contract.decode_event::<TakeOrderFilter>("TakeOrder", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}

pub async fn get_context_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<ContextFilter> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.context_filter().filter;

    let option_log = get_matched_log(receipt, filter).await;

    match option_log {
        Some(log) => {
            let event = contract.decode_event::<ContextFilter>("Context", log.topics, log.data)?;
            return Ok(event);
        }
        None => return Err(Error::msg("event not found")),
    }
}

pub async fn _get_context_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx_hash: &TxHash,
) -> Result<Vec<ContextFilter>> {
    let receipt = get_pending_tx(tx_hash).await?;
    let filter: Filter = contract.clone().context_filter().filter;

    let option_logs = get_matched_logs(receipt, filter).await;

    match option_logs {
        Some(logs) => {
            let mut events: Vec<ContextFilter> = Vec::new();

            for log in logs {
                let event: ContextFilter =
                    contract.decode_event::<ContextFilter>("Context", log.topics, log.data)?;

                events.push(event);
            }

            return Ok(events);
        }
        None => return Err(Error::msg("events not found")),
    }
}
