use crate::config::Metaboard;
use crate::yaml::{require_hash, require_string, YamlError, YamlParsableHash};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use url::{ParseError, Url};

// Wrapper type just for YAML parsing
#[derive(Clone)]
pub struct YamlMetaboard(Metaboard);

impl YamlMetaboard {
    pub fn metaboard(&self) -> &Metaboard {
        &self.0
    }

    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }
}

impl YamlParsableHash for YamlMetaboard {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, YamlMetaboard>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let metaboards_hash = require_hash(
            &document_read,
            Some("metaboards"),
            Some("missing field: metaboards".to_string()),
        )?;

        metaboards_hash
            .iter()
            .map(|(key_yaml, metaboard_yaml)| {
                let metaboard_key = key_yaml.as_str().unwrap_or_default().to_string();

                let url = YamlMetaboard::validate_url(&require_string(
                    metaboard_yaml,
                    None,
                    Some(format!(
                        "metaboard value must be a string for key: {metaboard_key}"
                    )),
                )?)?;

                Ok((metaboard_key, YamlMetaboard(url)))
            })
            .collect()
    }
}

impl From<Metaboard> for YamlMetaboard {
    fn from(value: Metaboard) -> Self {
        YamlMetaboard(value)
    }
}
impl From<YamlMetaboard> for Metaboard {
    fn from(value: YamlMetaboard) -> Self {
        value.0
    }
}
