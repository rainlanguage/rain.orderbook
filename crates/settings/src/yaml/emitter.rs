use super::{
    context::Context, FieldErrorKind, YamlError, YamlParsableHash, YamlParsableString,
    YamlParseableValue,
};
use crate::{
    accounts::AccountCfg, local_db_remotes::LocalDbRemoteCfg, local_db_sync::LocalDbSyncCfg,
    metaboard::MetaboardCfg, remote_networks::RemoteNetworksCfg, remote_tokens::RemoteTokensCfg,
    sentry::Sentry, spec_version::SpecVersion, subgraph::SubgraphCfg, ChartCfg, DeployerCfg,
    DeploymentCfg, GuiCfg, NetworkCfg, OrderCfg, OrderbookCfg, ScenarioCfg, TokenCfg,
};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml, StrictYamlEmitter};

pub fn validate_and_emit_documents(
    documents: &[Arc<RwLock<StrictYaml>>],
    context: Option<&Context>,
) -> Result<String, YamlError> {
    validate_hash_section::<OrderCfg>(documents, context)?;
    validate_hash_section::<ScenarioCfg>(documents, context)?;
    validate_hash_section::<DeploymentCfg>(documents, context)?;
    validate_hash_section::<NetworkCfg>(documents, context)?;
    validate_hash_section::<SubgraphCfg>(documents, context)?;
    validate_hash_section::<MetaboardCfg>(documents, context)?;
    validate_hash_section::<TokenCfg>(documents, context)?;
    validate_hash_section::<OrderbookCfg>(documents, context)?;
    validate_hash_section::<DeployerCfg>(documents, context)?;

    ChartCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    RemoteNetworksCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    AccountCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    LocalDbRemoteCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    LocalDbSyncCfg::parse_all_from_yaml(documents.to_vec(), context)?;

    GuiCfg::parse_from_yaml_optional(documents.to_vec(), context)?;
    RemoteTokensCfg::parse_from_yaml_optional(documents.to_vec(), context)?;

    validate_string_field::<SpecVersion>(documents)?;
    validate_optional_string_field::<Sentry>(documents)?;

    emit_documents(documents)
}

fn validate_hash_section<T: YamlParsableHash>(
    documents: &[Arc<RwLock<StrictYaml>>],
    context: Option<&Context>,
) -> Result<(), YamlError> {
    match T::parse_all_from_yaml(documents.to_vec(), context) {
        Ok(_) => Ok(()),
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(_),
            ..
        }) => Ok(()),
        Err(e) => Err(e),
    }
}

fn validate_string_field<T: YamlParsableString>(
    documents: &[Arc<RwLock<StrictYaml>>],
) -> Result<(), YamlError> {
    for document in documents {
        match T::parse_from_yaml(document.clone()) {
            Ok(_) => return Ok(()),
            Err(YamlError::Field {
                kind: FieldErrorKind::Missing(_),
                ..
            }) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn validate_optional_string_field<T: YamlParsableString>(
    documents: &[Arc<RwLock<StrictYaml>>],
) -> Result<(), YamlError> {
    for document in documents {
        T::parse_from_yaml_optional(document.clone())?;
    }
    Ok(())
}

fn emit_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<String, YamlError> {
    let mut merged_hash = Hash::new();

    for document in documents {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        if let StrictYaml::Hash(ref hash) = *document_read {
            for (key, value) in hash {
                merged_hash.insert(key.clone(), value.clone());
            }
        }
    }

    let merged_doc = StrictYaml::Hash(merged_hash);
    let mut out_str = String::new();
    let mut emitter = StrictYamlEmitter::new(&mut out_str);
    emitter.dump(&merged_doc)?;

    let out_str = if out_str.starts_with("---") {
        out_str.trim_start_matches("---").trim_start().to_string()
    } else {
        out_str
    };

    Ok(out_str)
}
