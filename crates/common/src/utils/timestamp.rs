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

#[cfg(test)]
mod tests {
    use core::num::IntErrorKind;

    use super::*;

    #[test]
    fn test_format_bigint_timestamp_display_err_parse_int_err() {
        let timestamp = "".to_string();
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::Empty
        ));

        let timestamp = "171502440000000000.0".to_string();
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::InvalidDigit
        ));

        let timestamp = format!("{}", i128::from(i64::MAX) + 1);
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::PosOverflow
        ));

        let timestamp = format!("{}", i128::from(i64::MIN) - 1);
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::NegOverflow
        ));
    }

    #[test]
    fn test_format_bigint_timestamp_display_err_invalid_timestamp() {
        // Test case for timestamp that would result in days < i32::MIN
        let timestamp = format!("{}", i64::MIN);
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MIN
        ));

        // Test case for timestamp that would result in days > i32::MAX
        // This is a timestamp that would be far in the future
        let timestamp = format!("{}", i64::MAX);
        let result = format_bigint_timestamp_display(timestamp);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MAX
        ));
    }

    #[test]
    fn test_format_bigint_timestamp_display_ok() {
        let timestamp = "1746537612".to_string();
        std::env::set_var("TZ", "CET"); // Required to make local timezone deterministic
        let result = format_bigint_timestamp_display(timestamp);
        assert_eq!(result, Ok("2025-05-06 03:20:12 PM".to_string()));

        let timestamp = "970676358".to_string();
        std::env::set_var("TZ", "EST"); // Required to make local timezone deterministic
        let result = format_bigint_timestamp_display(timestamp);
        assert_eq!(result, Ok("2000-10-04 06:19:18 PM".to_string()));
    }
}
