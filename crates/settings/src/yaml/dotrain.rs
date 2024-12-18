use super::*;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DotrainYaml {
    pub document: Arc<RwLock<StrictYaml>>,
}

impl YamlParsable for DotrainYaml {
    fn new(source: String, validate: bool) -> Result<Self, YamlError> {
        let docs = StrictYamlLoader::load_from_str(&source)?;
        if docs.is_empty() {
            return Err(YamlError::EmptyFile);
        }
        let doc = docs[0].clone();
        let document = Arc::new(RwLock::new(doc));

        if validate {}

        Ok(DotrainYaml { document })
    }
}

impl DotrainYaml {}
