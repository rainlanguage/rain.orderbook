use crate::yaml::{FieldErrorKind, YamlError};
use url::{ParseError as UrlParseError, Url};

pub fn parse_positive_u32(value: &str, field: &str, location: String) -> Result<u32, YamlError> {
    let parsed: u32 = value
        .parse()
        .map_err(|e: std::num::ParseIntError| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: field.to_string(),
                reason: e.to_string(),
            },
            location: location.clone(),
        })?;
    if parsed == 0 {
        return Err(YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: field.to_string(),
                reason: "must be a positive integer".to_string(),
            },
            location,
        });
    }
    Ok(parsed)
}

pub fn parse_positive_u64(value: &str, field: &str, location: String) -> Result<u64, YamlError> {
    let parsed: u64 = value
        .parse()
        .map_err(|e: std::num::ParseIntError| YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: field.to_string(),
                reason: e.to_string(),
            },
            location: location.clone(),
        })?;
    if parsed == 0 {
        return Err(YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: field.to_string(),
                reason: "must be a positive integer".to_string(),
            },
            location,
        });
    }
    Ok(parsed)
}

pub fn parse_url(value: &str, field: &str, location: String) -> Result<Url, YamlError> {
    Url::parse(value).map_err(|e: UrlParseError| YamlError::Field {
        kind: FieldErrorKind::InvalidValue {
            field: field.to_string(),
            reason: e.to_string(),
        },
        location,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_positive_u32_valid() {
        let v = parse_positive_u32("42", "f", "loc".to_string()).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn test_parse_positive_u32_zero_and_non_numeric() {
        let err = parse_positive_u32("0", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let err = parse_positive_u32("abc", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        // negative string path should also error via parse failure
        let err = parse_positive_u32("-1", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_parse_positive_u64_valid() {
        let v = parse_positive_u64("42", "f", "loc".to_string()).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn test_parse_positive_u64_zero_and_non_numeric() {
        let err = parse_positive_u64("0", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let err = parse_positive_u64("abc", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let err = parse_positive_u64("-1", "f", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_parse_url_valid_and_invalid() {
        let u = parse_url("http://example.com", "url", "loc".to_string()).unwrap();
        assert_eq!(u.as_str(), "http://example.com/");

        let err = parse_url("::not a url::", "url", "loc".to_string()).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }
}
