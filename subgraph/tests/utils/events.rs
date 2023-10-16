use crate::generated::{AddOrderFilter, OrderBook};
use crate::generated::{ERC20Mock, TransferFilter};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, PendingTransaction, Provider},
    signers::Wallet,
    types::{Filter, Log, Topic, TransactionReceipt, TxHash, ValueOrArray},
};

async fn _get_matched_log(tx: PendingTransaction<'_, Http>, filter: Filter) -> Option<Log> {
    let tx_receipt: TransactionReceipt = tx.await.expect("Failed to get the receipt").unwrap();

    let topic = filter.topics[0].clone().expect("failed to get the topic");

    let topic_hash = _extract_topic_hash(topic).expect("cannot get the hash from the topic");

    for log in tx_receipt.logs.iter() {
        if let Some(first_topic) = log.topics.get(0) {
            if first_topic == &topic_hash {
                return Some(log.clone());
            }
        }
    }

    None
}

/// Try to extract the hash value from a Topic (ValueOrArray) type
fn _extract_topic_hash(topic: ValueOrArray<Option<TxHash>>) -> Option<TxHash> {
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
    tx: PendingTransaction<'_, Http>,
) -> TransferFilter {
    let filter = contract.transfer_filter().filter;

    let log = _get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<TransferFilter>("Transfer", log.topics, log.data)
        .expect("cannot decode the event");
}

pub async fn _get_add_order_event(
    contract: OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: PendingTransaction<'_, Http>,
) -> AddOrderFilter {
    let filter: Filter = contract.clone().add_order_filter().filter;

    let log = _get_matched_log(tx, filter)
        .await
        .expect("there is no topic matched in the transaction");

    return contract
        .decode_event::<AddOrderFilter>("AddOrder", log.topics, log.data)
        .expect("cannot decode the event");
}
