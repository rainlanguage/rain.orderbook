//! Interpreter host abstractions used by the Virtual Raindex.

mod revm;

pub use revm::{EvalOutcome, InterpreterHost, RevmInterpreterHost};
