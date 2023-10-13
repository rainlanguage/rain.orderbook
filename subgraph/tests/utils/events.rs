use ethers::{abi::AbiEncode, prelude::*};
use ethers::{
    providers::PendingTransaction,
    types::{Log, Topic, TransactionReceipt, TxHash, ValueOrArray},
};

// use crate::abigen::ERC20Mock::{ERC20Mock, TransferFilter};
use crate::generated::{ERC20Mock, TransferFilter};

// use crate::generated::{erc20_mock, ERC20MockEvents};

use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::Wallet,
};

pub fn get_matched_log(logs: Vec<Log>, topic: ValueOrArray<Option<TxHash>>) -> Option<Log> {
    let topic_hash = extract_topic_hash(topic).expect("cannot get the hash from the topic");

    for log in logs.iter() {
        if let Some(first_topic) = log.topics.get(0) {
            if first_topic == &topic_hash {
                return Some(log.clone());
            }
        }
    }

    None
}

/// Transfer event decode from receipt
///
/// ## Arguments
///
/// * `contract` -  The contract that contain the event/filter/topic
///
/// ## Returns
///
/// The struct value
pub async fn get_transfer_event(
    contract: ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    tx: PendingTransaction<'_, Http>,
) -> () {
    let tx_receipt: TransactionReceipt = tx.await.expect("Failed to get the receipt").unwrap();

    let topic: ValueOrArray<Option<TxHash>> =
        contract.transfer_filter().filter.topics[0].clone().unwrap();

    let log = get_matched_log(tx_receipt.logs.clone(), topic)
        .expect("there is no topic matched in the transaction");

    // contract.transfer_filter()
    // TransferFilter::default()
    // ERC20MockEvents::TransferFilter((TransferFilter:))

    let aver = contract
        .decode_event::<TransferFilter>("Transfer", log.topics, log.data)
        .unwrap();

    println!("aver: {:?}", aver);
    println!("aver: {:?}", aver.value);
}

/// Try to extract the hash value from a Topic (ValueOrArray) type
fn extract_topic_hash(topic: ValueOrArray<Option<TxHash>>) -> Option<TxHash> {
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
