use crate::meta::{TryDecodeRainlangSource, TryDecodeRainlangSourceError};
use rain_orderbook_subgraph_client::types::order_detail;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderDetailExtended {
    pub order: order_detail::Order,
    pub rainlang: Option<String>,
}

impl TryFrom<order_detail::Order> for OrderDetailExtended {
    type Error = TryDecodeRainlangSourceError;

    fn try_from(val: order_detail::Order) -> Result<Self, TryDecodeRainlangSourceError> {
        let rainlang = val
            .clone()
            .meta
            .map(|meta| meta.try_decode_rainlangsource())
            .transpose()?;

        Ok(Self {
            order: val,
            rainlang: rainlang,
        })
    }
}
