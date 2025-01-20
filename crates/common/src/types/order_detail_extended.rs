use crate::meta::{TryDecodeRainlangSource, TryDecodeRainlangSourceError};
use rain_orderbook_subgraph_client::types::common::Order;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderDetailExtended {
    #[typeshare(typescript(type = "OrderSubgraph"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "OrderSubgraph"))]
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
