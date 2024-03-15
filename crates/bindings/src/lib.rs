use alloy_sol_types::sol;

sol!(
    #![sol(all_derives = true)]
    #[derive(serde::Serialize, serde::Deserialize)]
    IOrderBookV3, "../../out/IOrderBookV3.sol/IOrderBookV3.json"
);

sol!(
    #![sol(all_derives = true)]
    IERC20, "../../out/IERC20.sol/IERC20.json"
);

sol!(
    #![sol(all_derives = true)]
    ERC20, "../../out/ERC20.sol/ERC20.json"
);
