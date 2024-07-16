use rain_interpreter_eval::error::ForkCallError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FailedQuote {
    #[error("Order does not exist")]
    NonExistent,
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
}
