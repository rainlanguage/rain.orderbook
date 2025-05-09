use chrono::{DateTime, Local};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
#[test]
fn test_format_bigint_timestamp_display_ok() {
    // Required to make local timezone deterministic
    // NOTE: Setting TZ affects global state.
    std::env::set_var("TZ", "CET");

    let timestamp = "1746537612".to_string();
    let result = format_bigint_timestamp_display(timestamp.clone());
    assert_eq!(result, Ok("2025-05-06 03:20:12 PM".to_string()));

    let timestamp_i64 = timestamp.parse::<i64>().unwrap();
    let result = format_timestamp_display(timestamp_i64);
    assert_eq!(result, Ok("2025-05-06 03:20:12 PM".to_string()));

    // Required to make local timezone deterministic
    // NOTE: Setting TZ affects global state.
    std::env::set_var("TZ", "EST");

    let timestamp = "970676358".to_string();
    let result = format_bigint_timestamp_display(timestamp.clone());
    assert_eq!(result, Ok("2000-10-04 06:19:18 PM".to_string()));

    let timestamp_i64 = timestamp.parse::<i64>().unwrap();
    let result = format_timestamp_display(timestamp_i64);
    assert_eq!(result, Ok("2000-10-04 06:19:18 PM".to_string()));

    // Test earliest valid timestamp (close to Unix epoch minimum)
    // January 1, 1970 (plus some seconds to be safe)
    let earliest_valid = "86400".to_string(); // 1 day after epoch
    let result = format_bigint_timestamp_display(earliest_valid.clone());
    assert!(result.is_ok());

    // Test latest reasonably valid timestamp
    // December 31, 2099 (arbitrary future date that should be valid)
    let future_valid = "4102444800".to_string();
    let result = format_bigint_timestamp_display(future_valid.clone());
    assert!(result.is_ok());
}
