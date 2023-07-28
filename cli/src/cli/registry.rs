
use clap::ValueEnum;

/// # Ethereum
/// Network details for Ethereum mainnet.
#[derive(Debug,Clone)]
pub struct Ethereum { 
    pub rpc_url : String,
    pub subgraph_url : String ,
    pub scan_base_uri : String ,
    pub chain_id : String ,  
    pub block_scanner_api : String, 
    pub block_scanner_key : String
}  

impl Ethereum {
    pub fn new(rpc_url: String, block_scanner_key: String) -> Ethereum {
        return Ethereum{
            rpc_url : rpc_url ,
            subgraph_url: String::from("https://api.thegraph.com/subgraphs/name/rainprotocol/interpreter-registry-ethereum") ,
            scan_base_uri : String::from("https://etherscan.io/") ,
            chain_id :  String::from("1") , 
            block_scanner_api : String::from("https://api.etherscan.io/") ,
            block_scanner_key : block_scanner_key
        };
    }
}  

/// # Polygon
/// Network details for Polygon mainnet.
#[derive(Debug,Clone)]
pub struct Polygon {
    pub rpc_url : String,
    pub subgraph_url : String ,
    pub scan_base_uri : String ,
    pub chain_id : String ,
    pub block_scanner_api : String, 
    pub block_scanner_key : String
}  

impl Polygon {
    pub fn new(rpc_url: String, block_scanner_key: String) -> Polygon {
        return Polygon{
            rpc_url : rpc_url ,
            subgraph_url: String::from("https://api.thegraph.com/subgraphs/name/rainprotocol/interpreter-registry-polygon") ,
            scan_base_uri : String::from("https://polygonscan.com/") ,
            chain_id :  String::from("137") ,
            block_scanner_api : String::from("https://api.polygonscan.com/") ,
            block_scanner_key : block_scanner_key
        };
    }
}  

/// # Mumbai
/// Network details for Polygon testnet (Mumbai).
#[derive(Debug,Clone)]
pub struct Mumbai {
    pub rpc_url : String,
    pub subgraph_url : String ,
    pub scan_base_uri : String ,
    pub chain_id : String ,
    pub block_scanner_api : String, 
    pub block_scanner_key : String
}  

impl Mumbai {
    pub fn new(rpc_url: String, block_scanner_key: String) -> Mumbai {
        return Mumbai{
            rpc_url : rpc_url ,
            subgraph_url: String::from("https://api.thegraph.com/subgraphs/name/rainprotocol/interpreter-registry") ,
            scan_base_uri : String::from("https://mumbai.polygonscan.com/") ,
            chain_id :  String::from("80001") ,
            block_scanner_api : String::from("https://api-testnet.polygonscan.com/") ,
            block_scanner_key : block_scanner_key
        };
    }
}   

/// # Fuji
/// Network details for Avalanche testnet (Fuji).
#[derive(Debug,Clone)]
pub struct Fuji {
    pub rpc_url : String,
    pub subgraph_url : String ,
    pub scan_base_uri : String ,
    pub chain_id : String , 
    pub block_scanner_api : String, 
    pub block_scanner_key : String
}  

impl Fuji {
    pub fn new(rpc_url: String, block_scanner_key: String) -> Fuji {
        return Fuji{
            rpc_url : rpc_url ,
            subgraph_url: String::from("") ,
            scan_base_uri : String::from("https://testnet.snowtrace.io/") ,
            chain_id :  String::from("43113") , 
            block_scanner_api : String::from("https://api-testnet.snowtrace.io/api") ,
            block_scanner_key : block_scanner_key
        };
    }
}  

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

/// # RainNetworks
/// Value Enums representing supported networks for cross deploying contracts.
#[derive(Debug)]
 #[derive(Clone)]
pub enum RainNetworks{
    Ethereum(Ethereum),
    Polygon(Polygon),
    Mumbai(Mumbai),
    Fuji(Fuji)
}

