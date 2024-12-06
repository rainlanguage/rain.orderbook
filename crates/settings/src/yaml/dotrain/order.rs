use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct IOYaml {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderYaml {
    pub inputs: Vec<IOYaml>,
    pub outputs: Vec<IOYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbook: Option<String>,
}
impl OrderYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut orders = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("orders"),
            Some("missing field: orders".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let order = OrderYaml {
                inputs: require_vec(
                    value,
                    "inputs",
                    Some(format!("inputs list missing in order: {}", key)),
                )?
                .iter()
                .enumerate()
                .map(|(i, input)| {
                    Ok::<IOYaml, YamlError>(IOYaml {
                        token: require_string(
                            input,
                            Some("token"),
                            Some(format!(
                                "token string missing in input index: {} in order: {}",
                                i, key
                            )),
                        )?,
                        vault_id: optional_string(input, "vault-id"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
                outputs: require_vec(
                    value,
                    "outputs",
                    Some(format!("outputs list missing in order: {}", key)),
                )?
                .iter()
                .enumerate()
                .map(|(i, output)| {
                    Ok::<IOYaml, YamlError>(IOYaml {
                        token: require_string(
                            output,
                            Some("token"),
                            Some(format!(
                                "token string missing in output index: {} in order: {}",
                                i, key
                            )),
                        )?,
                        vault_id: optional_string(output, "vault-id"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
                deployer: optional_string(value, "deployer"),
                orderbook: optional_string(value, "orderbook"),
            };
            orders.insert(key.to_string(), order);
        }
        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orders_errors() {
        let yaml = r#"
test: test
"#;
        let error = OrderYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: orders".to_string())
        );

        let yaml = r#"
orders:
    order1:
"#;
        let error = OrderYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("inputs list missing in order: order1".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - test: test
"#;
        let error = OrderYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token string missing in input index: 0 in order: order1".to_string()
            )
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
"#;
        let error = OrderYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("outputs list missing in order: order1".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - test: test
"#;
        let error = OrderYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token string missing in output index: 0 in order: order1".to_string()
            )
        );
    }
}
