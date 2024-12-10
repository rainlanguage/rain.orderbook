use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NetworkYaml {
    pub document: Arc<RwLock<StrictYaml>>,
    pub rpc: String,
    pub chain_id: String,
    pub label: Option<String>,
    pub network_id: Option<String>,
    pub currency: Option<String>,
}
impl NetworkYaml {
    pub fn validate(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, NetworkYaml>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

        let networks_hash = require_hash(
            &document_read,
            Some("networks"),
            Some("missing field: networks".to_string()),
        )?;

        let networks = networks_hash
            .into_iter()
            .map(|(key, value)| {
                let key = key.as_str().unwrap_or_default().to_string();
                let network = NetworkYaml {
                    document: document.clone(),
                    rpc: require_string(
                        &value,
                        Some("rpc"),
                        Some(format!("rpc string missing in network: {key}")),
                    )?,
                    chain_id: require_string(
                        &value,
                        Some("chain-id"),
                        Some(format!("chain-id string missing in network: {key}")),
                    )?,
                    label: optional_string(&value, "label"),
                    network_id: optional_string(&value, "network-id"),
                    currency: optional_string(&value, "currency"),
                };
                Ok((key, network))
            })
            .collect::<Result<HashMap<_, _>, YamlError>>()?;

        Ok(networks)
    }

    pub fn update_rpc(
        document: &Arc<RwLock<StrictYaml>>,
        key: &str,
        rpc: &str,
    ) -> Result<(), YamlError> {
        let mut document = document.write().map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut networks)) =
                document_hash.get_mut(&StrictYaml::String("networks".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut network)) =
                    networks.get_mut(&StrictYaml::String(key.to_string()))
                {
                    network[&StrictYaml::String("rpc".to_string())] =
                        StrictYaml::String(rpc.to_string());
                } else {
                    return Err(YamlError::ParseError(format!(
                        "network {} missing in networks",
                        key
                    )));
                }
            } else {
                return Err(YamlError::ParseError(
                    "networks missing in document".to_string(),
                ));
            }
        } else {
            return Err(YamlError::ParseError("document parse error".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }

    #[test]
    fn test_validation() {
        let yaml = r#"
test: test
"#;
        let error = NetworkYaml::validate(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = NetworkYaml::validate(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc string missing in network: mainnet".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = NetworkYaml::validate(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("chain-id string missing in network: mainnet".to_string())
        );
    }
}
