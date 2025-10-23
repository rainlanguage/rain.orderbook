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
