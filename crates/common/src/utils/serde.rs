// Shared serde helpers for the common crate.
// These are intentionally generic and not tied to local_db specifics.

/// Accepts 0/1 integers, booleans, or strings ("true"/"false"/"1"/"0")
/// and deserializes them into a Rust `bool`.
/// Useful for sources like SQLite that may emit 0/1 rather than true/false.
pub fn bool_from_int_or_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected};
    struct BoolOrIntVisitor;

    impl<'de> serde::de::Visitor<'de> for BoolOrIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or 0/1 integer")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::invalid_value(
                    Unexpected::Unsigned(v),
                    &"0 or 1 for boolean",
                )),
            }
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::invalid_value(
                    Unexpected::Signed(v),
                    &"0 or 1 for boolean",
                )),
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v.to_ascii_lowercase().as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(E::invalid_value(
                    Unexpected::Str(v),
                    &"'true'/'false' or '1'/'0'",
                )),
            }
        }
    }

    deserializer.deserialize_any(BoolOrIntVisitor)
}

#[cfg(test)]
mod tests {
    use super::bool_from_int_or_bool;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Wrap {
        #[serde(deserialize_with = "bool_from_int_or_bool")]
        b: bool,
    }

    #[test]
    fn deserializes_bool_values() {
        let t: Wrap = serde_json::from_str("{\"b\": true}").unwrap();
        assert!(t.b);
        let f: Wrap = serde_json::from_str("{\"b\": false}").unwrap();
        assert!(!f.b);
    }

    #[test]
    fn deserializes_int_values() {
        let t: Wrap = serde_json::from_str("{\"b\": 1}").unwrap();
        assert!(t.b);
        let f: Wrap = serde_json::from_str("{\"b\": 0}").unwrap();
        assert!(!f.b);
    }

    #[test]
    fn deserializes_string_values() {
        let t1: Wrap = serde_json::from_str("{\"b\": \"true\"}").unwrap();
        assert!(t1.b);
        let t2: Wrap = serde_json::from_str("{\"b\": \"1\"}").unwrap();
        assert!(t2.b);
        let f1: Wrap = serde_json::from_str("{\"b\": \"false\"}").unwrap();
        assert!(!f1.b);
        let f2: Wrap = serde_json::from_str("{\"b\": \"0\"}").unwrap();
        assert!(!f2.b);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(serde_json::from_str::<Wrap>("{\"b\": 2}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": -1}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": \"yes\"}").is_err());
        assert!(serde_json::from_str::<Wrap>("{\"b\": \"maybe\"}").is_err());
    }
}

