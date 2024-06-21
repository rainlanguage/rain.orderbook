use alloy_sol_types::sol;

sol!(
    #![sol(all_derives = true)]
    #[derive(serde::Serialize, serde::Deserialize)]
    IOrderBookV4, "../../out/IOrderBookV4.sol/IOrderBookV4.json"
);

sol!(
    #![sol(all_derives = true)]
    IERC20, "../../out/IERC20.sol/IERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);
