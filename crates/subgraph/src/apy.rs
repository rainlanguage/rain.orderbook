use crate::{
    types::common::Erc20, vol::VaultVolume, OrderbookSubgraphClient, OrderbookSubgraphClientError,
};
use alloy::primitives::{I256, U256};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use typeshare::typeshare;

pub const YEAR: u64 = 60 * 60 * 24 * 365;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultAPY {
    pub id: String,
    pub token: Erc20,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,
    #[typeshare(typescript(type = "string"))]
    pub capital: U256,
    #[typeshare(typescript(type = "string"))]
    pub apy: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct APYTimeframe {
    pub start_time: u64,
    pub end_time: u64,
}

/// Given a subgraph and an order id and optionally a timeframe, will fetch data
/// and calculates the APY for each of the order's vaults
pub async fn get_order_vaults_apy(
    subgraph_url: Url,
    order_id: &str,
    timeframe: Option<APYTimeframe>,
) -> Result<Vec<VaultAPY>, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(subgraph_url);
    let vols = if let Some(timeframe) = timeframe {
        client
            .order_vaults_volume(
                cynic::Id::new(order_id),
                Some(timeframe.start_time),
                Some(timeframe.end_time),
            )
            .await?
    } else {
        let order = client.order_detail(cynic::Id::new(order_id)).await?;
        let mut vols: Vec<VaultVolume> = vec![];
        for vault in &order.inputs {
            if !vols
                .iter()
                .any(|v| v.id == vault.vault_id.0 && v.token.address.0 == vault.token.address.0)
            {
                let total_in = U256::from_str(&vault.total_volume_in.0)?;
                let total_out = U256::from_str(&vault.total_volume_out.0)?;
                vols.push(VaultVolume {
                    id: vault.vault_id.0.clone(),
                    token: vault.token.clone(),
                    total_in,
                    total_out,
                    total_vol: total_in.saturating_add(total_out),
                    net_vol: I256::from_raw(total_in).saturating_sub(I256::from_raw(total_out)),
                    all_time_vol_in: total_in,
                    all_time_vol_out: total_out,
                })
            }
        }
        for vault in &order.outputs {
            if !vols
                .iter()
                .any(|v| v.id == vault.vault_id.0 && v.token.address.0 == vault.token.address.0)
            {
                let total_in = U256::from_str(&vault.total_volume_in.0)?;
                let total_out = U256::from_str(&vault.total_volume_out.0)?;
                vols.push(VaultVolume {
                    id: vault.vault_id.0.clone(),
                    token: vault.token.clone(),
                    total_in,
                    total_out,
                    total_vol: total_in.saturating_add(total_out),
                    net_vol: I256::from_raw(total_in).saturating_sub(I256::from_raw(total_out)),
                    all_time_vol_in: total_in,
                    all_time_vol_out: total_out,
                })
            }
        }
        vols
    };

    let mut vaults_apy: Vec<VaultAPY> = vec![];
    for vol in vols {
        let vault_bal_change = client
            .first_day_vault_balance_change(
                cynic::Id::new(&vol.id),
                timeframe.map(|v| v.start_time),
            )
            .await?;
        let capital = U256::from_str(
            &vault_bal_change
                .as_ref()
                .map(|v| v.old_vault_balance.0.clone())
                .unwrap_or("0".to_string()),
        )?;
        let start = u64::from_str(
            &timeframe
                .map(|v: APYTimeframe| v.start_time.to_string())
                .unwrap_or(
                    vault_bal_change
                        .as_ref()
                        .map(|v| v.timestamp.0.clone())
                        .unwrap_or("0".to_string()),
                ),
        )?;
        let end = timeframe
            .map(|v| v.end_time)
            .unwrap_or(chrono::Utc::now().timestamp() as u64);
        let apy = if capital.is_zero() || start == 0 {
            0_i64
        } else {
            let change_ratio = i64::try_from(
                vol.net_vol
                    .saturating_mul(I256::from_raw(U256::from(10000)))
                    .saturating_div(I256::from_raw(capital)),
            )? / 10000;
            let time_to_year_ratio = ((end - start) / YEAR) as i64;
            (change_ratio * time_to_year_ratio) * 100
        };
        vaults_apy.push(VaultAPY {
            id: vol.id.clone(),
            token: vol.token.clone(),
            start_time: timeframe.map(|v| v.start_time),
            end_time: timeframe.map(|v| v.end_time),
            net_vol: vol.net_vol,
            apy,
            capital,
        });
    }

    Ok(vaults_apy)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::types::common::{
//         BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
//         Transaction, VaultBalanceChangeVault,
//     };
//     use alloy::primitives::{Address, B256};

//     // helper function that returns mocked sg response in json
//     fn get_sg_response() -> Value {
//         let io = IO::default();
//         let order = OrderV3 {
//             validInputs: vec![io.clone()],
//             validOutputs: vec![io.clone()],
//             ..Default::default()
//         };
//         json!({
//             "data": {
//                 "order": {
//                     "id": encode_prefixed(B256::random()),
//                     "owner": encode_prefixed(order.owner),
//                     "orderHash": encode_prefixed(B256::random()),
//                     "orderBytes": encode_prefixed(order.abi_encode()),
//                     "outputs": [{
//                         "id": encode_prefixed(B256::random()),
//                         "balance": "0",
//                         "vaultId": io.vaultId.to_string(),
//                         "token": {
//                             "name": "T1",
//                             "symbol": "T1",
//                             "id": encode_prefixed(io.token),
//                             "address": encode_prefixed(io.token),
//                             "decimals": io.decimals.to_string(),
//                         },
//                         "orderbook": { "id": encode_prefixed(B256::random()) },
//                         "owner": encode_prefixed(order.owner),
//                         "ordersAsOutput": [],
//                         "ordersAsInput": [],
//                         "balanceChanges": []
//                         "totalVolumeIn": "1",
//                         "totalVolumeOut": "1",
//                     }],
//                     "inputs": [{
//                         "id": encode_prefixed(B256::random()),
//                         "balance": "0",
//                         "vaultId": io.vaultId.to_string(),
//                         "token": {
//                             "name": "T2",
//                             "symbol": "T2",
//                             "id": encode_prefixed(io.token),
//                             "address": encode_prefixed(io.token),
//                             "decimals": io.decimals.to_string(),
//                         },
//                         "orderbook": { "id": encode_prefixed(B256::random()) },
//                         "owner": encode_prefixed(order.owner),
//                         "ordersAsOutput": [],
//                         "ordersAsInput": [],
//                         "balanceChanges": [],
//                         "totalVolumeIn": "1",
//                         "totalVolumeOut": "1",
//                     }],
//                     "orderbook": {
//                         "id": encode_prefixed(B256::random()),
//                     },
//                     "meta": null,
//                     "active": true,
//                     "timestampAdded": "0",
//                     "addEvents": [{
//                         "transaction": {
//                             "id": encode_prefixed(B256::random()),
//                             "blockNumber": "0",
//                             "timestamp": "0",
//                             "from": encode_prefixed(alloy::primitives::Address::random())
//                         }
//                     }],
//                     "trades": []
//                 }
//             }
//         })
//     }

//     #[test]
//     fn test_get_order_vaults_vol() {}
// }
