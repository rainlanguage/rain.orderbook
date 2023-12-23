use alloy_sol_types::sol;
use ethers::contract::abigen;

sol!(IOrderBookV3, "src/abi/IOrderBookV3.json");

abigen!(IParserV1, "src/abi/IParserV1.json");
abigen!(
    IExpressionDeployerV3,
    r#"[
        function iInterpreter() public view returns(address)
        function iStore() public view returns(address)
        function iParser() public view returns(address)
    ]"#,
);
