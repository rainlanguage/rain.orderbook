#[cfg(target_family = "wasm")]
mod js_api {
    use super::super::common::{
        AddOrder, BigInt, Bytes, ClearBounty, Deposit, Erc20, Order, OrderAsIO,
        OrderStructPartialTrade, TradeVaultBalanceChange, Transaction, Vault, VaultBalanceChange,
        VaultBalanceChangeVault, Withdrawal,
    };
    use rain_orderbook_bindings::impl_wasm_traits;
    use serde_wasm_bindgen::{from_value, to_value};
    use wasm_bindgen::{convert::*, describe::WasmDescribe};
    use wasm_bindgen::{
        describe::{inform, WasmDescribeVector, VECTOR},
        JsValue, UnwrapThrowExt,
    };

    impl_wasm_traits!(Order);
    impl_wasm_traits!(Vault);
    impl_wasm_traits!(AddOrder);
    impl_wasm_traits!(OrderAsIO);
    impl_wasm_traits!(VaultBalanceChangeVault);
    impl_wasm_traits!(VaultBalanceChange);
    impl_wasm_traits!(Withdrawal);
    impl_wasm_traits!(TradeVaultBalanceChange);
    impl_wasm_traits!(Deposit);
    impl_wasm_traits!(ClearBounty);
    impl_wasm_traits!(OrderStructPartialTrade);
    impl_wasm_traits!(Erc20);
    impl_wasm_traits!(Transaction);
    impl_wasm_traits!(BigInt);
    impl_wasm_traits!(Bytes);
}
