use crate::remote::tokens::{RemoteTokensError, Tokens};
use crate::yaml::context::Context;
use crate::yaml::{
    default_document, optional_string, FieldErrorKind, YamlError, YamlParseableValue,
};
use crate::{NetworkCfg, TokenCfg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RemoteTokensCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RemoteTokensCfg);

impl RemoteTokensCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseRemoteTokensError> {
        Url::parse(value).map_err(ParseRemoteTokensError::UrlParseError)
    }

    pub async fn fetch_tokens(
        networks: &HashMap<String, NetworkCfg>,
        remote_tokens: RemoteTokensCfg,
    ) -> Result<HashMap<String, TokenCfg>, ParseRemoteTokensError> {
        let mut tokens: HashMap<String, TokenCfg> = HashMap::new();

        let tokens_res = reqwest::get(remote_tokens.url.to_string())
            .await?
            .json::<Tokens>()
            .await?;

        for token in &tokens_res.tokens {
            let token_cfg = token
                .clone()
                .try_into_token_cfg(networks, remote_tokens.document.clone())?;

            if tokens.contains_key(&token_cfg.key) {
                return Err(ParseRemoteTokensError::ConflictingTokens(
                    token_cfg.key.clone(),
                ));
            }
            tokens.insert(token_cfg.key.clone(), token_cfg);
        }

        Ok(tokens)
    }
}

impl YamlParseableValue for RemoteTokensCfg {
    fn parse_from_yaml(
        _: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<RemoteTokensCfg, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<Option<RemoteTokensCfg>, YamlError> {
        let mut url: Option<Url> = None;
        let mut document_index: usize = 0;

        for (index, document) in documents.iter().enumerate() {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(value) = optional_string(&document_read, "using-tokens-from") {
                if let Some(url) = url {
                    return Err(YamlError::KeyShadowing(
                        url.to_string(),
                        "using-tokens-from".to_string(),
                    ));
                }

                let validated_url =
                    RemoteTokensCfg::validate_url(&value).map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "url".to_string(),
                            reason: e.to_string(),
                        },
                        location: "using-tokens-from".to_string(),
                    })?;
                url = Some(validated_url);
                document_index = index;
            }
        }

        Ok(url.map(|url| RemoteTokensCfg {
            url,
            document: documents[document_index].clone(),
        }))
    }
}

impl Default for RemoteTokensCfg {
    fn default() -> Self {
        RemoteTokensCfg {
            document: default_document(),
            url: Url::parse("http://example.com").unwrap(),
        }
    }
}
impl PartialEq for RemoteTokensCfg {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

#[derive(Error, Debug)]
pub enum ParseRemoteTokensError {
    #[error("Conflicting remote token in response, a token with key '{0}' already exists")]
    ConflictingTokens(String),
    #[error(transparent)]
    UrlParseError(ParseError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    RemoteTokensError(#[from] RemoteTokensError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_parse_remote_tokens_from_yaml() {
        let yaml = r#"
using-tokens-from: test
"#;
        let error =
            RemoteTokensCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "url".to_string(),
                    reason: "relative URL without a base".to_string(),
                },
                location: "using-tokens-from".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_remote_tokens_from_multiple_files() {
        let yaml_one = r#"
using-tokens-from: http://test.com
"#;
        let yaml_two = r#"
using-tokens-from: http://test.com
"#;
        let error = RemoteTokensCfg::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing(
                "http://test.com/".to_string(),
                "using-tokens-from".to_string()
            )
        );
    }
}
