use crate::meta::{TryDecodeRainlangSource, TryDecodeRainlangSourceError};
#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_subgraph_client::types::common::Order;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct OrderDetailExtended {
    pub order: Order,
    pub rainlang: Option<String>,
}

impl TryFrom<Order> for OrderDetailExtended {
    type Error = TryDecodeRainlangSourceError;

    fn try_from(val: Order) -> Result<Self, TryDecodeRainlangSourceError> {
        let rainlang = val
            .clone()
            .meta
            .map(|meta| meta.try_decode_rainlangsource())
            .transpose()?;

        Ok(Self {
            order: val,
            rainlang,
        })
    }
}

#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(OrderDetailExtended);
