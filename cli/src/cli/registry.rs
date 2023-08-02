
use clap::ValueEnum;

use ethers::contract::abigen ; 

/// # RainNetworkOptions
/// Enum representing options for supported networks for cross deploying contracts.
 #[derive(Debug)]
 #[derive(Copy,Clone,ValueEnum)]
pub enum RainNetworkOptions{
    Ethereum,
    Polygon,
    Mumbai,
    Fuji
}  


abigen!(IOrderBookV2, "src/cli/abis/IOrderBookV2.json"); 

abigen!(
    IERC20,
    r#"[
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval(address indexed owner, address indexed spender, uint256 value)
    ]"#,
);


abigen!(IParserV1, "src/cli/abis/IParserV1.json");  