//! Error types surfaced by the Virtual Raindex crate.

use std::{error::Error, fmt};

use alloy::primitives::{Address, B256};
use rain_math_float::{Float, FloatError};
use serde::{Deserialize, Serialize};

/// Convenience alias for fallible Virtual Raindex results.
pub type Result<T> = std::result::Result<T, RaindexError>;

/// Errors that can occur while working with the Virtual Raindex.
#[derive(Debug)]
pub enum RaindexError {
    /// Placeholder error used for mutations and features that are not yet implemented.
    Unimplemented(&'static str),
    /// Error bubbling up from Float math helpers.
    Float(FloatError),
    /// Raised when the interpreter host cannot resolve bytecode for a given address.
    MissingBytecode {
        address: Address,
        kind: BytecodeKind,
    },
    /// Raised when provided bytecode cannot be decoded into binary form.
    InvalidBytecodeEncoding {
        address: Address,
        kind: BytecodeKind,
    },
    /// Raised when attempting to insert conflicting bytecode for an address.
    BytecodeCollision {
        address: Address,
        kind: BytecodeKind,
    },
    /// Wrapper for REVM execution failures.
    RevmExecution(String),
    /// Raised when a referenced order hash cannot be resolved from state.
    OrderNotFound { order_hash: B256 },
    /// Raised when an input IO index is outside the order's valid inputs array.
    InvalidInputIndex { index: usize, len: usize },
    /// Raised when an output IO index is outside the order's valid outputs array.
    InvalidOutputIndex { index: usize, len: usize },
    /// Raised when token decimals are required but missing from virtual state.
    TokenDecimalMissing { token: Address },
    /// Raised when a quote attempts to use the same token for input and output.
    TokenSelfTrade,
    /// Raised when the take orders config contains no orders.
    NoOrders,
    /// Raised when the maximum input for take orders is zero.
    ZeroMaximumInput,
    /// Raised when take orders mix different input or output tokens.
    TokenMismatch,
    /// Raised when the total input from take orders is less than the configured minimum.
    MinimumInputNotMet { minimum: Float, actual: Float },
}

impl From<FloatError> for RaindexError {
    fn from(value: FloatError) -> Self {
        Self::Float(value)
    }
}

impl fmt::Display for RaindexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaindexError::Unimplemented(feature) => write!(f, "{feature} is not implemented"),
            RaindexError::Float(err) => write!(f, "float error: {err}"),
            RaindexError::MissingBytecode { address, kind } => {
                write!(f, "missing {kind} bytecode for address {address}")
            }
            RaindexError::InvalidBytecodeEncoding { address, kind } => {
                write!(f, "invalid {kind} bytecode encoding for address {address}")
            }
            RaindexError::BytecodeCollision { address, kind } => {
                write!(
                    f,
                    "conflicting {kind} bytecode already cached for address {address}"
                )
            }
            RaindexError::RevmExecution(reason) => write!(f, "revm execution failed: {reason}"),
            RaindexError::OrderNotFound { order_hash } => {
                write!(f, "order {order_hash:?} not found in virtual state")
            }
            RaindexError::InvalidInputIndex { index, len } => {
                write!(f, "input IO index {index} out of bounds (len {len})")
            }
            RaindexError::InvalidOutputIndex { index, len } => {
                write!(f, "output IO index {index} out of bounds (len {len})")
            }
            RaindexError::TokenDecimalMissing { token } => {
                write!(f, "missing token decimals for {token}")
            }
            RaindexError::TokenSelfTrade => write!(f, "token self trade is not allowed"),
            RaindexError::NoOrders => write!(f, "take orders requires at least one order"),
            RaindexError::ZeroMaximumInput => {
                write!(f, "take orders maximum input must be positive")
            }
            RaindexError::TokenMismatch => {
                write!(f, "all take orders must share the same input/output tokens")
            }
            RaindexError::MinimumInputNotMet { minimum, actual } => {
                let min = minimum
                    .format()
                    .unwrap_or_else(|_| "<format error>".to_string());
                let act = actual
                    .format()
                    .unwrap_or_else(|_| "<format error>".to_string());
                write!(
                    f,
                    "take orders minimum input {min} not satisfied (actual {act})"
                )
            }
        }
    }
}

impl Error for RaindexError {}

/// Identifies the type of bytecode requested from the [CodeCache].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BytecodeKind {
    Interpreter,
    Store,
}

impl fmt::Display for BytecodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytecodeKind::Interpreter => write!(f, "interpreter"),
            BytecodeKind::Store => write!(f, "store"),
        }
    }
}
