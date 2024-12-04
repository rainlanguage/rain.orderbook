use alloy::primitives::{I256, U256};
use rain_orderbook_subgraph_client::types::common::Erc20;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultVolume {
    id: String,
    token: Erc20,
    #[typeshare(typescript(type = "string"))]
    total_in: U256,
    #[typeshare(typescript(type = "string"))]
    total_out: U256,
    #[typeshare(typescript(type = "string"))]
    total_vol: U256,
    #[typeshare(typescript(type = "string"))]
    net_vol: I256,
}
