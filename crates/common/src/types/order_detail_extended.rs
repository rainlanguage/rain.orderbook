use crate::meta::{TryDecodeRainlangSource, TryDecodeRainlangSourceError};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailExtended {
    pub order: SgOrder,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub rainlang: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderDetailExtended);

impl TryFrom<SgOrder> for OrderDetailExtended {
    type Error = TryDecodeRainlangSourceError;

    fn try_from(val: SgOrder) -> Result<Self, TryDecodeRainlangSourceError> {
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
