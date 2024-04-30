use chrono::{DateTime, Local};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatTimestampDisplayError {
    #[error("Timestamp is invalid {0}")]
    InvalidTimestamp(i64),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

pub fn format_bigint_timestamp_display(
    timestamp: String,
) -> Result<String, FormatTimestampDisplayError> {
    let timestamp_i64 = timestamp.parse::<i64>()?;
    format_timestamp_display(timestamp_i64)
}

pub fn format_timestamp_display(timestamp: i64) -> Result<String, FormatTimestampDisplayError> {
    let timestamp_naive = DateTime::from_timestamp(timestamp, 0)
        .ok_or(FormatTimestampDisplayError::InvalidTimestamp(timestamp))?;

    let timestamp_display = timestamp_naive
        .with_timezone(&Local)
        .format("%Y-%m-%d %I:%M:%S %p")
        .to_string();

    Ok(timestamp_display)
}
