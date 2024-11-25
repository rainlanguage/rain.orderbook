use alloy::sol;

sol!(
    #![sol(all_derives = true)]
    #[derive(serde::Serialize, serde::Deserialize)]
    IOrderBookV4, "../../out/IOrderBookV4.sol/IOrderBookV4.json"
);

sol!(
    #![sol(all_derives = true)]
    #[derive(serde::Serialize, serde::Deserialize)]
    OrderBook, "../../out/OrderBook.sol/OrderBook.json"
);

sol!(
    #![sol(all_derives = true)]
    IERC20, "../../out/IERC20.sol/IERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);

#[cfg(target_family = "wasm")]
pub mod js_api;

#[cfg(target_family = "wasm")]
pub mod wasm_traits;
