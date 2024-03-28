#![allow(dead_code)]

use lazy_static::lazy_static;
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

lazy_static! {
  #[rustfmt::skip]
  pub static ref MIN_VALID_SETTINGS_KEYS: String =
    "networks:" + Key::Enter + Key::Tab +
      "polygon:" + Key::Enter + Key::Tab +
        "rpc: https://rpc.ankr.com/polygon" + Key::Enter +
        "chain-id: 137" + Key::Enter +
        "label: Polygon" + Key::Enter +
        "network-id: 137" + Key::Enter +
        "currency: MATIC" + Key::Enter + Key::Backspace + Key::Backspace +
    "subgraphs:" + Key::Enter + Key::Tab +
      "polygon: https://api.thegraph.com/subgraphs/name/siddharth2207/obv3subparser" + Key::Enter + Key::Backspace +
    "orderbooks:" + Key::Enter + Key::Tab +
      "polygonOB:" + Key::Enter + Key::Tab +
        "address: 0xDE5aBE2837bc042397D80E37fb7b2C850a8d5a6C" + Key::Enter +
        "network: polygon" + Key::Enter +
        "subgraph: polygon" + Key::Enter +
        "label: Polygon Orderbook";
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
