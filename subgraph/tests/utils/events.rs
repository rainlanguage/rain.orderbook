use crate::generated::{AddOrderFilter, AfterClearFilter, ClearFilter, OrderBook};
use crate::generated::{ERC20Mock, TransferFilter};
use crate::generated::{NewExpressionFilter, RainterpreterExpressionDeployer};
use ethers::providers::Middleware;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, PendingTransaction, Provider},
    signers::Wallet,
    types::{Filter, Log, Topic, TransactionReceipt, TxHash},
};

use super::get_provider;

async fn get_matched_log(tx: &PendingTransaction<'_, Http>, filter: Filter) -> Option<Log> {
    let tx_hash = tx.tx_hash().clone();
    println!("tx_hash: {:?}", tx_hash);

    let provider = get_provider().await.unwrap();


    let tx_receipt: TransactionReceipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .expect("Failed to get the receipt")
        .expect("receipt empty - maybe not minted yet");

    let topic_hash = extract_topic_hash(filter).expect("cannot get the topic hash");

    for log in tx_receipt.logs.iter() {
        if let Some(first_topic) = log.topics.get(0) {
            if first_topic == &topic_hash {
                return Some(log.clone());
            }
        }
    }

    None
}

/// Get all the logs in a transaction for a given filter.
async fn get_matched_logs(tx: &PendingTransaction<'_, Http>, filter: Filter) -> Option<Vec<Log>> {
    let tx_hash = tx.tx_hash().clone();

    let provider = get_provider().await.unwrap();

    let tx_receipt: TransactionReceipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .expect("Failed to get the receipt")
        .unwrap();

    let topic_hash = extract_topic_hash(filter).expect("cannot get the topic hash");

    let mut logs: Vec<Log> = Vec::new();

    for log in tx_receipt.logs.iter() {
        if let Some(first_topic) = log.topics.get(0) {
            if first_topic == &topic_hash {
                logs.push(log.clone())
            }
        }
    }

    if logs.len() > 0 {
        return Some(logs);
    }

    None
}

/// Try to extract the hash value from a Topic (ValueOrArray) type
// fn extract_topic_hash(topic: ValueOrArray<Option<TxHash>>) -> Option<TxHash> {
fn extract_topic_hash(filter: Filter) -> Option<TxHash> {
    let topic = filter.topics[0]
        .clone()
        .expect("failed to get the topic from filter");

    match topic {
        Topic::Value(Some(data)) => Some(data),
        Topic::Array(topic) => {
            if let Some(data) = topic.get(0) {
                return data.clone();
            } else {
                return None;
            }
        }
        _ => None,
    }
}

pub async fn _get_transfer_event(
    contract: ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> TransferFilter {
    let filter = contract.transfer_filter().filter;

    let log = get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<TransferFilter>("Transfer", log.topics, log.data)
        .expect("cannot decode the event");
}

pub async fn get_add_order_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> AddOrderFilter {
    let filter: Filter = contract.clone().add_order_filter().filter;

    let log = get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<AddOrderFilter>("AddOrder", log.topics, log.data)
        .expect("cannot decode the event");
}

pub async fn get_add_order_events(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> Vec<AddOrderFilter> {
    let filter: Filter = contract.clone().add_order_filter().filter;

    let logs = get_matched_logs(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    let mut events: Vec<AddOrderFilter> = Vec::new();

    for log in logs {
        let event: AddOrderFilter = contract
            .decode_event::<AddOrderFilter>("AddOrder", log.topics, log.data)
            .expect("cannot decode the event");

        events.push(event);
    }

    return events;
}

pub async fn get_new_expression_event(
    contract: RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> NewExpressionFilter {
    let filter: Filter = contract.clone().new_expression_filter().filter;

    let log = get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<NewExpressionFilter>("NewExpression", log.topics, log.data)
        .expect("cannot decode the event");
}

pub async fn get_clear_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> ClearFilter {
    let filter: Filter = contract.clear_filter().filter;

    let log = get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<ClearFilter>("Clear", log.topics, log.data)
        .expect("cannot decode the event");
}

pub async fn get_after_clear_event(
    contract: &OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: &PendingTransaction<'_, Http>,
) -> AfterClearFilter {
    let filter: Filter = contract.after_clear_filter().filter;

    let log = get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<AfterClearFilter>("AfterClear", log.topics, log.data)
        .expect("cannot decode the event");
}
