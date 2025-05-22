use chrono::DateTime;
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
    is_local: bool,
) -> Result<String, FormatTimestampDisplayError> {
    let timestamp_i64 = timestamp.parse::<i64>()?;
    if is_local {
        format_timestamp_display_local(timestamp_i64)
    } else {
        format_timestamp_display_utc(timestamp_i64)
    }
}

fn get_datetime_from_timestamp(
    timestamp: i64,
) -> Result<DateTime<chrono::Utc>, FormatTimestampDisplayError> {
    DateTime::from_timestamp(timestamp, 0)
        .ok_or(FormatTimestampDisplayError::InvalidTimestamp(timestamp))
}

pub fn format_timestamp_display_utc(timestamp: i64) -> Result<String, FormatTimestampDisplayError> {
    let timestamp_naive = get_datetime_from_timestamp(timestamp)?;
    Ok(timestamp_naive.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

pub fn format_timestamp_display_local(
    timestamp: i64,
) -> Result<String, FormatTimestampDisplayError> {
    let timestamp_naive = get_datetime_from_timestamp(timestamp)?;
    Ok(timestamp_naive
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %I:%M:%S %p UTC%:::z")
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::IntErrorKind;
    use std::env;

    #[test]
    fn utc_test_format_bigint_timestamp_display_err_parse_int_err() {
        let timestamp = "".to_string();
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::Empty
        ));

        let timestamp = "171502440000000000.0".to_string();
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::InvalidDigit
        ));

        let timestamp = format!("{}", i128::from(i64::MAX) + 1);
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::PosOverflow
        ));

        let timestamp = format!("{}", i128::from(i64::MIN) - 1);
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::NegOverflow
        ));
    }

    #[test]
    fn local_test_format_bigint_timestamp_display_err_parse_int_err() {
        env::set_var("TZ", "America/New_York");

        let timestamp = "".to_string();
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::Empty
        ));

        let timestamp = "171502440000000000.0".to_string();
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::InvalidDigit
        ));

        let timestamp = format!("{}", i128::from(i64::MAX) + 1);
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::PosOverflow
        ));

        let timestamp = format!("{}", i128::from(i64::MIN) - 1);
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::ParseIntError(err)) if err.kind() == &IntErrorKind::NegOverflow
        ));

        env::remove_var("TZ");
    }

    #[test]
    fn utc_test_format_bigint_timestamp_display_err_invalid_timestamp() {
        let timestamp = format!("{}", i64::MIN);
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MIN
        ));

        let timestamp = format!("{}", i64::MAX);
        let result = format_bigint_timestamp_display(timestamp, false);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MAX
        ));
    }

    #[test]
    fn local_test_format_bigint_timestamp_display_err_invalid_timestamp() {
        env::set_var("TZ", "America/New_York");

        let timestamp = format!("{}", i64::MIN);
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MIN
        ));

        let timestamp = format!("{}", i64::MAX);
        let result = format_bigint_timestamp_display(timestamp, true);
        assert!(matches!(
            result,
            Err(FormatTimestampDisplayError::InvalidTimestamp(t)) if t == i64::MAX
        ));

        env::remove_var("TZ");
    }

    #[test]
    fn utc_test_format_bigint_timestamp_display_ok() {
        let timestamp = "1746537612".to_string();
        let result = format_bigint_timestamp_display(timestamp.clone(), false);
        assert_eq!(result, Ok("2025-05-06 13:20:12 UTC".to_string()));

        let timestamp = "970676358".to_string();
        let result = format_bigint_timestamp_display(timestamp.clone(), false);
        assert_eq!(result, Ok("2000-10-04 16:19:18 UTC".to_string()));

        let earliest_valid = "86400".to_string();
        let result = format_bigint_timestamp_display(earliest_valid.clone(), false);
        assert!(result.is_ok());

        let future_valid = "4102444800".to_string();
        let result = format_bigint_timestamp_display(future_valid.clone(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn local_test_format_bigint_timestamp_display_ok() {
        env::set_var("TZ", "America/New_York");

        let timestamp = "1746537612".to_string();
        let result = format_bigint_timestamp_display(timestamp.clone(), true);
        assert_eq!(result, Ok("2025-05-06 09:20:12 AM UTC-04".to_string()));

        let timestamp = "970676358".to_string();
        let result = format_bigint_timestamp_display(timestamp.clone(), true);
        assert_eq!(result, Ok("2000-10-04 12:19:18 PM UTC-04".to_string()));

        let earliest_valid = "86400".to_string();
        let result = format_bigint_timestamp_display(earliest_valid.clone(), true);
        assert!(result.is_ok());

        let future_valid = "4102444800".to_string();
        let result = format_bigint_timestamp_display(future_valid.clone(), true);
        assert!(result.is_ok());

        env::remove_var("TZ");
    }
}
