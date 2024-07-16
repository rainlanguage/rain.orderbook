use crate::error::FailedQuote;
use alloy_primitives::U256;
use rain_orderbook_bindings::IOrderBookV4::quoteReturn;
use serde::{Deserialize, Serialize};

pub type QuoteResult = Result<OrderQuote, FailedQuote>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderQuote {
    pub max_output: U256,
    pub ratio: U256,
}

impl From<quoteReturn> for OrderQuote {
    fn from(v: quoteReturn) -> Self {
        Self {
            max_output: v.outputMax,
            ratio: v.ioRatio,
        }
    }
}
