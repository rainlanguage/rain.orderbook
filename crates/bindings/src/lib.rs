use alloy_sol_types::sol;

sol!(
    #![sol(all_derives = true)]
    IOrderBookV3, "../../out/IOrderBookV3.sol/IOrderBookV3.json"
);

sol!(
    #![sol(all_derives = true)]
    IERC20, "../../out/ERC20.sol/ERC20.json"
);
