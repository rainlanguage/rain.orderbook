use crate::error::FailedQuote;
use alloy_primitives::{Address, U256};
use rain_orderbook_bindings::IOrderBookV4::{quoteReturn, Quote};
use serde::{Deserialize, Serialize};

pub type QuoteResult = Result<OrderQuote, FailedQuote>;

/// Holds quoted order max output and ratio
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

/// A quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteTarget {
    pub quote: Quote,
    pub orderbook: Address,
}
