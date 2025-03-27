use alloy::primitives::Address;

use rain_orderbook_common::erc20::ERC20;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

mod erc20;
