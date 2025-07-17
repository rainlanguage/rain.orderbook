use alloy::primitives::{
    utils::{parse_units, ParseUnits},
    U256,
};
use rain_orderbook_app_settings::gui::{DepositValidationCfg, FieldValueValidationCfg};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuiValidationError {
    #[error("Invalid number for {name}: {value}")]
    InvalidNumber { name: String, value: String },

    #[error("{name} value {value} is less than minimum {minimum}")]
    BelowMinimum {
        name: String,
        value: String,
        minimum: String,
    },

    #[error("{name} value {value} is less than or equal to {exclusive_minimum}")]
    BelowExclusiveMinimum {
        name: String,
        value: String,
        exclusive_minimum: String,
    },

    #[error("{name} value {value} is greater than maximum {maximum}")]
    AboveMaximum {
        name: String,
        value: String,
        maximum: String,
    },

    #[error("{name} value {value} is greater than or equal to {exclusive_maximum}")]
    AboveExclusiveMaximum {
        name: String,
        value: String,
        exclusive_maximum: String,
    },

    #[error("{name} value {value} is not a multiple of {multiple_of}")]
    NotMultipleOf {
        name: String,
        value: String,
        multiple_of: String,
    },

    #[error("{name} length {length} is less than minimum {minimum}")]
    StringTooShort {
        name: String,
        length: u32,
        minimum: u32,
    },

    #[error("{name} length {length} exceeds maximum {maximum}")]
    StringTooLong {
        name: String,
        length: u32,
        maximum: u32,
    },

    #[error("{name} value '{value}' is not a valid boolean (must be '1' or '0')")]
    InvalidBoolean { name: String, value: String },
}

pub fn validate_field_value(
    field_name: &str,
    value: &str,
    validation: &FieldValueValidationCfg,
) -> Result<(), GuiValidationError> {
    match validation {
        FieldValueValidationCfg::Number {
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
            multiple_of,
            decimals,
        } => validate_number(
            field_name,
            value,
            *decimals,
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
            multiple_of,
        ),
        FieldValueValidationCfg::String {
            min_length,
            max_length,
        } => validate_string(field_name, value, min_length, max_length),
        FieldValueValidationCfg::Boolean => validate_boolean(field_name, value),
    }
}

pub fn validate_deposit_amount(
    token_name: &str,
    amount: &str,
    validation: &DepositValidationCfg,
    decimals: u8,
) -> Result<(), GuiValidationError> {
    validate_number(
        token_name,
        amount,
        decimals,
        &validation.minimum,
        &validation.exclusive_minimum,
        &validation.maximum,
        &validation.exclusive_maximum,
        &validation.multiple_of,
    )
}

fn validate_number(
    name: &str,
    value: &str,
    decimals: u8,
    minimum: &Option<String>,
    exclusive_minimum: &Option<String>,
    maximum: &Option<String>,
    exclusive_maximum: &Option<String>,
    multiple_of: &Option<String>,
) -> Result<(), GuiValidationError> {
    if value.is_empty() {
        return Err(GuiValidationError::InvalidNumber {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    let parsed_value =
        parse_units(value, decimals).map_err(|_| GuiValidationError::InvalidNumber {
            name: name.to_string(),
            value: value.to_string(),
        })?;

    if let Some(min) = minimum {
        let min_value =
            parse_units(min, decimals).map_err(|_| GuiValidationError::InvalidNumber {
                name: name.to_string(),
                value: min.clone(),
            })?;
        if parsed_value < min_value {
            return Err(GuiValidationError::BelowMinimum {
                name: name.to_string(),
                value: value.to_string(),
                minimum: min.clone(),
            });
        }
    }

    if let Some(exclusive_min) = exclusive_minimum {
        let exclusive_min_value = parse_units(exclusive_min, decimals).map_err(|_| {
            GuiValidationError::InvalidNumber {
                name: name.to_string(),
                value: exclusive_min.clone(),
            }
        })?;
        if parsed_value <= exclusive_min_value {
            return Err(GuiValidationError::BelowExclusiveMinimum {
                name: name.to_string(),
                value: value.to_string(),
                exclusive_minimum: exclusive_min.clone(),
            });
        }
    }

    if let Some(max) = maximum {
        let max_value =
            parse_units(max, decimals).map_err(|_| GuiValidationError::InvalidNumber {
                name: name.to_string(),
                value: max.clone(),
            })?;
        if parsed_value > max_value {
            return Err(GuiValidationError::AboveMaximum {
                name: name.to_string(),
                value: value.to_string(),
                maximum: max.clone(),
            });
        }
    }

    if let Some(exclusive_max) = exclusive_maximum {
        let exclusive_max_value = parse_units(exclusive_max, decimals).map_err(|_| {
            GuiValidationError::InvalidNumber {
                name: name.to_string(),
                value: exclusive_max.clone(),
            }
        })?;
        if parsed_value >= exclusive_max_value {
            return Err(GuiValidationError::AboveExclusiveMaximum {
                name: name.to_string(),
                value: value.to_string(),
                exclusive_maximum: exclusive_max.clone(),
            });
        }
    }

    if let Some(multiple) = multiple_of {
        let multiple_value =
            parse_units(multiple, decimals).map_err(|_| GuiValidationError::InvalidNumber {
                name: name.to_string(),
                value: multiple.clone(),
            })?;
        let multiple_u256 = match multiple_value {
            ParseUnits::U256(v) => v,
            _ => {
                return Err(GuiValidationError::InvalidNumber {
                    name: name.to_string(),
                    value: multiple.clone(),
                })
            }
        };
        let parsed_u256 = match parsed_value {
            ParseUnits::U256(v) => v,
            _ => {
                return Err(GuiValidationError::InvalidNumber {
                    name: name.to_string(),
                    value: value.to_string(),
                })
            }
        };
        if multiple_u256 > U256::ZERO && parsed_u256 % multiple_u256 != U256::ZERO {
            return Err(GuiValidationError::NotMultipleOf {
                name: name.to_string(),
                value: value.to_string(),
                multiple_of: multiple.clone(),
            });
        }
    }

    Ok(())
}

fn validate_string(
    name: &str,
    value: &str,
    min_length: &Option<u32>,
    max_length: &Option<u32>,
) -> Result<(), GuiValidationError> {
    let length = value.len() as u32;

    if let Some(min) = min_length {
        if length < *min {
            return Err(GuiValidationError::StringTooShort {
                name: name.to_string(),
                length,
                minimum: *min,
            });
        }
    }

    if let Some(max) = max_length {
        if length > *max {
            return Err(GuiValidationError::StringTooLong {
                name: name.to_string(),
                length,
                maximum: *max,
            });
        }
    }

    Ok(())
}

fn validate_boolean(name: &str, value: &str) -> Result<(), GuiValidationError> {
    match value {
        "1" | "0" => Ok(()),
        _ => Err(GuiValidationError::InvalidBoolean {
            name: name.to_string(),
            value: value.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::gui::FieldValueValidationCfg;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_validate_number_minimum() {
        let result = validate_number(
            "Test Field",
            "5",
            18,
            &Some("10".to_string()),
            &None,
            &None,
            &None,
            &None,
        );
        match &result {
            Err(GuiValidationError::BelowMinimum {
                name,
                value,
                minimum,
            }) => {
                assert_eq!(name, "Test Field");
                assert_eq!(value, "5");
                assert_eq!(minimum, "10");
            }
            _ => panic!("Expected BelowMinimum error"),
        }

        let result = validate_number(
            "Test Field",
            "10",
            18,
            &Some("10".to_string()),
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "15",
            18,
            &Some("10".to_string()),
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_number_exclusive_minimum() {
        let result = validate_number(
            "Price",
            "10",
            18,
            &None,
            &Some("10".to_string()),
            &None,
            &None,
            &None,
        );
        match &result {
            Err(GuiValidationError::BelowExclusiveMinimum {
                name,
                value,
                exclusive_minimum,
            }) => {
                assert_eq!(name, "Price");
                assert_eq!(value, "10");
                assert_eq!(exclusive_minimum, "10");
            }
            _ => panic!("Expected BelowExclusiveMinimum error"),
        }

        let result = validate_number(
            "Price",
            "10.1",
            18,
            &None,
            &Some("10".to_string()),
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_number_maximum() {
        let result = validate_number(
            "Amount",
            "101",
            18,
            &None,
            &None,
            &Some("100".to_string()),
            &None,
            &None,
        );
        match &result {
            Err(GuiValidationError::AboveMaximum {
                name,
                value,
                maximum,
            }) => {
                assert_eq!(name, "Amount");
                assert_eq!(value, "101");
                assert_eq!(maximum, "100");
            }
            _ => panic!("Expected AboveMaximum error"),
        }

        let result = validate_number(
            "Amount",
            "100",
            18,
            &None,
            &None,
            &Some("100".to_string()),
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "99.9",
            18,
            &None,
            &None,
            &Some("100".to_string()),
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_number_exclusive_maximum() {
        let result = validate_number(
            "Token Amount",
            "100",
            18,
            &None,
            &None,
            &None,
            &Some("100".to_string()),
            &None,
        );
        match &result {
            Err(GuiValidationError::AboveExclusiveMaximum {
                name,
                value,
                exclusive_maximum,
            }) => {
                assert_eq!(name, "Token Amount");
                assert_eq!(value, "100");
                assert_eq!(exclusive_maximum, "100");
            }
            _ => panic!("Expected AboveExclusiveMaximum error"),
        }

        let result = validate_number(
            "Token Amount",
            "99.999",
            18,
            &None,
            &None,
            &None,
            &Some("100".to_string()),
            &None,
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_number_multiple_of() {
        let result = validate_number(
            "Step Size",
            "7",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("5".to_string()),
        );
        match &result {
            Err(GuiValidationError::NotMultipleOf {
                name,
                value,
                multiple_of,
            }) => {
                assert_eq!(name, "Step Size");
                assert_eq!(value, "7");
                assert_eq!(multiple_of, "5");
            }
            _ => panic!("Expected NotMultipleOf error"),
        }

        let result = validate_number(
            "Step Size",
            "10",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("5".to_string()),
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Step Size",
            "0",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("5".to_string()),
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Price Step",
            "1.05",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("0.01".to_string()),
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Price Step",
            "1.055",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("0.01".to_string()),
        );
        match &result {
            Err(GuiValidationError::NotMultipleOf { .. }) => {}
            _ => panic!("Expected NotMultipleOf error for decimal mismatch"),
        }
    }

    #[wasm_bindgen_test]
    fn test_validate_number_combined_constraints() {
        let result = validate_number(
            "Complex Field",
            "50",
            18,
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
            &Some("10".to_string()),
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Complex Field",
            "5",
            18,
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
            &Some("5".to_string()),
        );
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        let result = validate_number(
            "Complex Field",
            "105",
            18,
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
            &Some("5".to_string()),
        );
        assert!(matches!(
            result,
            Err(GuiValidationError::AboveMaximum { .. })
        ));

        let result = validate_number(
            "Complex Field",
            "53",
            18,
            &Some("10".to_string()),
            &None,
            &Some("100".to_string()),
            &None,
            &Some("10".to_string()),
        );
        assert!(matches!(
            result,
            Err(GuiValidationError::NotMultipleOf { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_number_parsing() {
        let result = validate_number("Test Field", "100.5", 18, &None, &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "0.000001",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "123456789.123456789",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Test Field",
            "not a number",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        match &result {
            Err(GuiValidationError::InvalidNumber { name, value }) => {
                assert_eq!(name, "Test Field");
                assert_eq!(value, "not a number");
            }
            _ => panic!("Expected InvalidNumber error"),
        }

        let result = validate_number(
            "Test Field",
            "12.34.56",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidNumber { .. })
        ));

        let result = validate_number("Test Field", "", 18, &None, &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidNumber { .. })
        ));

        let result = validate_number("Test Field", "1e18", 18, &None, &None, &None, &None, &None);
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidNumber { .. })
        ))
    }

    #[wasm_bindgen_test]
    fn test_validate_number_decimals() {
        let result = validate_number(
            "USDC Amount",
            "100.123456",
            6,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "ETH Amount",
            "1.123456789012345678",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "BTC Amount",
            "0.12345678",
            8,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_number_edge_cases() {
        let result = validate_number("Amount", "0", 18, &None, &None, &None, &None, &None);
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "0.000000000000000001",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "999999999999999999999999999",
            18,
            &None,
            &None,
            &None,
            &None,
            &None,
        );
        assert!(result.is_ok());

        let result = validate_number(
            "Amount",
            "100",
            18,
            &None,
            &None,
            &None,
            &None,
            &Some("0".to_string()),
        );
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_string_length() {
        let result = validate_string("Username", "hello", &Some(10), &None);
        match &result {
            Err(GuiValidationError::StringTooShort {
                name,
                length,
                minimum,
            }) => {
                assert_eq!(name, "Username");
                assert_eq!(*length, 5);
                assert_eq!(*minimum, 10);
            }
            _ => panic!("Expected StringTooShort error"),
        }

        let result = validate_string("Username", "hello world", &Some(5), &Some(20));
        assert!(result.is_ok());

        let result = validate_string("Username", "hello world", &Some(11), &None);
        assert!(result.is_ok());

        let result = validate_string("Description", &"a".repeat(100), &None, &Some(50));
        match &result {
            Err(GuiValidationError::StringTooLong {
                name,
                length,
                maximum,
            }) => {
                assert_eq!(name, "Description");
                assert_eq!(*length, 100);
                assert_eq!(*maximum, 50);
            }
            _ => panic!("Expected StringTooLong error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_validate_string_edge_cases() {
        let result = validate_string("Field", "", &None, &None);
        assert!(result.is_ok());

        let result = validate_string("Field", "", &Some(1), &None);
        assert!(matches!(
            result,
            Err(GuiValidationError::StringTooShort { .. })
        ));

        let result = validate_string("Field", "12345", &Some(5), &Some(5));
        assert!(result.is_ok());

        let result = validate_string("Field", "🦀", &Some(4), &None);
        assert!(result.is_ok());

        let result = validate_string("Field", "🦀", &Some(5), &None);
        assert!(matches!(
            result,
            Err(GuiValidationError::StringTooShort { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_boolean() {
        let result = validate_boolean("Enable Feature", "1");
        assert!(result.is_ok());

        let result = validate_boolean("Enable Feature", "0");
        assert!(result.is_ok());

        let result = validate_boolean("Enable Feature", "yes");
        match &result {
            Err(GuiValidationError::InvalidBoolean { name, value }) => {
                assert_eq!(name, "Enable Feature");
                assert_eq!(value, "yes");
            }
            _ => panic!("Expected InvalidBoolean error"),
        }

        let result = validate_boolean("Enable Feature", "True");
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "FALSE");
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "true");
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "false");
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));

        let result = validate_boolean("Enable Feature", "");
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_field_value_number() {
        let validation = FieldValueValidationCfg::Number {
            minimum: Some("10".to_string()),
            exclusive_minimum: None,
            maximum: Some("100".to_string()),
            exclusive_maximum: None,
            multiple_of: Some("5".to_string()),
            decimals: 18,
        };

        let result = validate_field_value("Price Field", "50", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Price Field", "5", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        let result = validate_field_value("Price Field", "102", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::AboveMaximum { .. })
        ));

        let result = validate_field_value("Price Field", "33", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::NotMultipleOf { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_field_value_string() {
        let validation = FieldValueValidationCfg::String {
            min_length: Some(3),
            max_length: Some(10),
        };

        let result = validate_field_value("Name Field", "hello", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Name Field", "hi", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::StringTooShort { .. })
        ));

        let result = validate_field_value("Name Field", "hello world!", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::StringTooLong { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_field_value_boolean() {
        let validation = FieldValueValidationCfg::Boolean;

        let result = validate_field_value("Toggle Field", "1", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Toggle Field", "0", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("Toggle Field", "maybe", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::InvalidBoolean { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_deposit_amount() {
        let validation = DepositValidationCfg {
            minimum: Some("0.01".to_string()),
            exclusive_minimum: None,
            maximum: Some("1000000".to_string()),
            exclusive_maximum: None,
            multiple_of: None,
        };

        let result = validate_deposit_amount("USDC", "100.50", &validation, 6);
        assert!(result.is_ok());

        let result = validate_deposit_amount("USDC", "0.001", &validation, 6);
        match &result {
            Err(GuiValidationError::BelowMinimum { name, .. }) => {
                assert_eq!(name, "USDC");
            }
            _ => panic!("Expected BelowMinimum error"),
        }

        let result = validate_deposit_amount("USDC", "1000001", &validation, 6);
        assert!(matches!(
            result,
            Err(GuiValidationError::AboveMaximum { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_field_value_custom_decimals() {
        // Test with 6 decimals (like USDC)
        let validation = FieldValueValidationCfg::Number {
            minimum: Some("0.01".to_string()),
            exclusive_minimum: None,
            maximum: Some("1000".to_string()),
            exclusive_maximum: None,
            multiple_of: Some("0.01".to_string()),
            decimals: 6,
        };

        let result = validate_field_value("USDC Amount", "100.12", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("USDC Amount", "0.005", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        let result = validate_field_value("USDC Amount", "100.125", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::NotMultipleOf { .. })
        ));

        // Test with 8 decimals (like BTC)
        let validation = FieldValueValidationCfg::Number {
            minimum: Some("0.00000001".to_string()),
            exclusive_minimum: None,
            maximum: Some("21000000".to_string()),
            exclusive_maximum: None,
            multiple_of: None,
            decimals: 8,
        };

        let result = validate_field_value("BTC Amount", "0.12345678", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("BTC Amount", "0.000000005", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        // Test with default decimals (18) when None
        let validation = FieldValueValidationCfg::Number {
            minimum: Some("0.000000000000000001".to_string()),
            exclusive_minimum: None,
            maximum: None,
            exclusive_maximum: None,
            multiple_of: None,
            decimals: 18,
        };

        let result = validate_field_value("ETH Amount", "0.000000000000000001", &validation);
        assert!(result.is_ok());

        let result = validate_field_value("ETH Amount", "0.0000000000000000005", &validation);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));
    }

    #[wasm_bindgen_test]
    fn test_validate_deposit_amount_with_different_decimals() {
        let validation = DepositValidationCfg {
            minimum: Some("0.000001".to_string()),
            exclusive_minimum: None,
            maximum: Some("21000000".to_string()),
            exclusive_maximum: None,
            multiple_of: None,
        };

        let result = validate_deposit_amount("Ethereum", "0.000000000000000001", &validation, 18);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        let result = validate_deposit_amount("Ethereum", "0.000001", &validation, 18);
        assert!(result.is_ok());

        let result = validate_deposit_amount("Bitcoin", "0.00000001", &validation, 8);
        assert!(matches!(
            result,
            Err(GuiValidationError::BelowMinimum { .. })
        ));

        let result = validate_deposit_amount("Bitcoin", "0.000001", &validation, 8);
        assert!(result.is_ok());
    }
}
