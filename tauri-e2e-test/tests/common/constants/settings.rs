#![allow(dead_code)]

use std::sync::OnceLock;
use thirtyfour::prelude::Key;

pub const MIN_VALID_SETTINGS: &str = "networks:
  polygon:
    rpc: https://rpc.ankr.com/polygon
    chain-id: 137
    label: Polygon
    network-id: 137
    currency: MATIC
subgraphs:
  polygon: https://api.thegraph.com/subgraphs/name/h20liquidity/polygon-0xc95a5f8e
orderbooks:
  polygonOB:
    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
    network: polygon
    subgraph: polygon
    label: Polygon Orderbook";

#[rustfmt::skip]
pub fn min_valid_settings_keys() -> &'static String {
  static MIN_VALID_SETTINGS_KEYS: OnceLock<String> = OnceLock::new();
  MIN_VALID_SETTINGS_KEYS.get_or_init(|| 
    "networks:" + Key::Enter + Key::Tab +
      "polygon:" + Key::Enter + Key::Tab +
        "rpc: https://rpc.ankr.com/polygon" + Key::Enter +
        "chain-id: 137" + Key::Enter +
        "label: Polygon" + Key::Enter +
        "network-id: 137" + Key::Enter +
        "currency: MATIC" + Key::Enter + Key::Backspace + Key::Backspace +
    "subgraphs:" + Key::Enter + Key::Tab +
      "polygon: https://api.thegraph.com/subgraphs/name/h20liquidity/polygon-0xc95a5f8e" + Key::Enter + Key::Backspace +
    "orderbooks:" + Key::Enter + Key::Tab +
      "polygonOB:" + Key::Enter + Key::Tab +
        "address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6" + Key::Enter +
        "network: polygon" + Key::Enter +
        "subgraph: polygon" + Key::Enter +
        "label: Polygon Orderbook"
  )
}

pub const VALID_SETTINGS_MULTIPLE: &str = "
networks: 
  polygon: 
    rpc: https://rpc.ankr.com/polygon 
    chain-id: 137 
    label: Polygon 
    network-id: 137 
    currency: MATIC 
  mainnet: 
    rpc: https://rpc.ankr.com/ethereum 
    chain-id: 1
    label: Ethereum
    network-id: 1
    currency: ETH 

subgraphs:
  polygon: https://api.thegraph.com/subgraphs/name/h20liquidity/polygon-0xc95a5f8e
  mainnet: https://api.thegraph.com/subgraphs/name/h20liquidity/ethereum-0xf1224a48

orderbooks:
  polygonOB:
    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
    network: polygon
    subgraph: polygon
    label: Polygon Orderbook
  polygonOB2:
    address: 0xabcdeabcdeabcdeabcdeabcdeabcdeabcdeabcde
    network: polygon
    subgraph: polygon
    label: Polygon Orderbook 2
  mainnetOB:
    address: 0xf1224A483ad7F1E9aA46A8CE41229F32d7549A74
    network: mainnet
    subgraph: mainnet
    label: Mainnet Orderbook
";

pub const VALID_WITH_NESTED_SCENARIO: &str = "
networks: 
  polygon: 
    rpc: https://rpc.ankr.com/polygon 
    chain-id: 137 
    label: Polygon 
    network-id: 137 
    currency: MATIC

subgraphs:
  polygon: https://api.thegraph.com/subgraphs/name/h20liquidity/polygon-0xc95a5f8e

orderbooks:
  polygonOB:
    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
    network: polygon
    subgraph: polygon
    label: Polygon Orderbook
  
scenarios:
  polygon:
    bindings:
      sub-parser: 0xc4b7A086FD25260461f7F50ac9D62Cb86006bbEB
      spread-multiplier: 101e16
    scenarios:
      sell:
        runs: 1
        bindings:
          binding1: 1
          binding2: 2
      buy:
        bindings:
          binding1: 3
          binding2: 4
";
