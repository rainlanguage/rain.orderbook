use schemars::JsonSchema;
use regex::Regex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;

/// Operands in the standard interpreter are `u16` values.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Operand {
    pub value: u16
}

/// Valid symbols in Rainlang are alpha prefixed alphanumeric kebab case.
pub const REGEX_RAIN_SYMBOL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z][0-9a-z-]*$").unwrap()
});

/// > An identifier in solidity has to start with a letter, a dollar-sign or an
/// > underscore and may additionally contain numbers after the first symbol.
/// https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityLexer.Identifier
pub const REGEX_SOLIDITY_IDENTIFIER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z$_][a-zA-Z0-9$_]*$").unwrap()
});

/// Strings in Rain are limited to printable ASCII chars and whitespace.
pub const REGEX_RAIN_STRING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\s!-~]*$").unwrap()
});

/// Titles in Rain are limited to printable ASCII chars and the space character.
/// The title MUST NOT begin or end with a space.
pub const REGEX_RAIN_TITLE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[!-~]([ -~]*[!-~]|[!-~]*)$").unwrap()
});

/// Rain symbols are a subset of kebab case.
#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RainSymbol{
    #[validate(regex(path = "REGEX_RAIN_SYMBOL", message = "Must be alphanumeric lower-kebab-case beginning with a letter.\n"))]
    pub value: String,
}

#[derive(Validate, JsonSchema, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RainTitle {
    #[validate(regex(path = "REGEX_RAIN_TITLE", message = "Must be alphanumeric ASCII letters and spaces.\n"))]
    pub value: String,
}

#[derive(Validate, JsonSchema, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct RainString {
    #[validate(regex(path = "REGEX_RAIN_STRING", message = "Must be printable ASCII characters and whitespace.\n"))]
    pub value: String,
}

pub type Description = RainString;

#[derive(Validate, JsonSchema, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SolidityIdentifier {
    #[validate(regex(path = "REGEX_SOLIDITY_IDENTIFIER", message = "Must be a valid Solidity identifier.\n"))]
    pub value: String,
}

#[cfg(test)]
mod test {
    use super::RainSymbol;
    use super::RainString;
    use super::RainTitle;
    use super::SolidityIdentifier;
    use validator::Validate;

    #[test]
    fn test_rain_symbol_validate() {
        // valids
        for i in ["a", "a-", "a-a", "a0"] {
            assert!(RainSymbol{ value: i.to_string()}.validate().is_ok(), "String '{}' considered invalid.", i);
        }

        // invalids
        for i in ["", "♥", "-", " ", "A", "A0", "a ", "0", "_", "0a", "0A", "\n", "\t", "\r"] {
            assert!(RainSymbol{ value: i.to_string()}.validate().is_err(), "String '{}' considered valid.", i);
        }
    }

    #[test]
    fn test_rain_title_validate() {
        // valids
        for i in ["a", "a-", "a-a", "a0", "a a", "-", "A", "A0", "0", "_", "0a", "0A",] {
            assert!(RainTitle{ value: i.to_string()}.validate().is_ok(), "String '{}' considered invalid.", i);
        }

        // invalids
        for i in ["", " ", " a", "a ", "♥", "\n", "\t", "\r"] {
            assert!(RainTitle{ value: i.to_string()}.validate().is_err(), "String '{}' considered valid.", i);
        }
    }

    #[test]
    fn test_rain_string_validate() {
        // valids
        for i in ["a", "aa", "aA", "aAa", "a0", "aa0", "aA0", "aA0a", "aA0a0", "", "a-", "a-a", "-", " ", "a ", "0", "_", "0a", "0A", "`", "```", "\n", "\t", "\r", ":"] {
            assert!(RainString{ value: i.to_string()}.validate().is_ok(), "String '{}' considered invalid.", i);
        }

        // invalids
        for i in ["♥", "∴"] {
            assert!(RainString{ value: i.to_string()}.validate().is_err(), "String '{}' considered valid.", i);
        }
    }

    #[test]
    fn test_solidity_identifier_validate() {
        // valids
        for i in ["A", "AA", "A0", "OrderBook", "$", "$$", "_", "__", "a", "aa", "a_", "A_", "a$", "A", "A$", "a0"] {
            assert!(SolidityIdentifier{ value: i.to_string()}.validate().is_ok(), "String '{}' considered invalid.", i);
        }

        // invalids
        for i in ["", "a-", "a-a", "♥", "-", " ", "a ", "0", "0a", "0A", "\n", "\t", "\r"] {
            assert!(SolidityIdentifier{ value: i.to_string()}.validate().is_err(), "String '{}' considered valid.", i);
        }
    }
}