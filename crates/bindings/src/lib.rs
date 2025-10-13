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
