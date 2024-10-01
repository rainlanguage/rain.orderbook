use crate::meta::{TryDecodeOrderMeta, TryDecodeRainlangSourceError};
use rain_orderbook_subgraph_client::types::common::Order;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderDetailExtended {
    pub order: Order,
    pub rainlang: Option<String>,
    pub dotrain: Option<String>,
}

impl TryFrom<Order> for OrderDetailExtended {
    type Error = TryDecodeRainlangSourceError;

    fn try_from(val: Order) -> Result<Self, TryDecodeRainlangSourceError> {
        let (rainlang, dotrain) = val
            .clone()
            .meta
            .map(|meta| meta.try_decode_meta())
            .transpose()?
            .map(|v| (Some(v.0), v.1))
            .unwrap_or((None, None));

        Ok(Self {
            order: val,
            rainlang,
            dotrain,
        })
    }
}
