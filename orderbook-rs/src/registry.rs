use alloy_sol_types::sol;
use ethers::contract::abigen;

sol!(IOrderBookV3, "../out/IOrderBookV3.sol/IOrderBookV3.json");

abigen!(IParserV1, "../out/IParserV1.sol/IParserV1.json");
abigen!(
    IExpressionDeployerV3,
    r#"[
        function iInterpreter() public view returns(address)
        function iStore() public view returns(address)
        function iParser() public view returns(address)
    ]"#,
);
