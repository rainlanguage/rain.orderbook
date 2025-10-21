use alloy::sol;

sol!(
    #![sol(all_derives = true, rpc)]
    #![sol(extra_derives(serde::Serialize, serde::Deserialize))]
    IOrderBookV5, "../../out/IOrderBookV5.sol/IOrderBookV5.json"
);

sol!(
    #![sol(all_derives = true)]
    #![sol(extra_derives(serde::Serialize, serde::Deserialize))]
    OrderBook, "../../out/OrderBook.sol/OrderBook.json"
);

sol!(
    #![sol(all_derives = true, rpc)]
    IERC20, "../../out/IERC20.sol/IERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    IInterpreterStoreV3, "../../out/IInterpreterStoreV3.sol/IInterpreterStoreV3.json"
);

pub mod provider;

#[cfg(target_family = "wasm")]
pub mod js_api;

#[cfg(target_family = "wasm")]
pub mod wasm_traits;

pub mod topics {
    use crate::{
        IInterpreterStoreV3::Set,
        IOrderBookV5::{
            AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
        },
        OrderBook::MetaV1_2,
    };
    use alloy::{primitives::B256, sol_types::SolEvent};

    pub fn orderbook_event_topics() -> Vec<B256> {
        vec![
            AddOrderV3::SIGNATURE_HASH,
            TakeOrderV3::SIGNATURE_HASH,
            WithdrawV2::SIGNATURE_HASH,
            DepositV2::SIGNATURE_HASH,
            RemoveOrderV3::SIGNATURE_HASH,
            ClearV3::SIGNATURE_HASH,
            AfterClearV2::SIGNATURE_HASH,
            MetaV1_2::SIGNATURE_HASH,
        ]
    }

    pub fn store_set_topics() -> Vec<B256> {
        vec![Set::SIGNATURE_HASH]
    }
}
