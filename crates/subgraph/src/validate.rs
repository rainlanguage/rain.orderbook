use cynic::{
    http::{CynicReqwestError, ReqwestBlockingExt},
    QueryBuilder,
};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SchemaError};
use std::fs::read_to_string;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaValidationError {
    #[error("undefined schema")]
    UndefinedSchema,
    #[error(transparent)]
    CynicReqwestError(#[from] CynicReqwestError),
    #[error(transparent)]
    ReadMainSchemaError(#[from] std::io::Error),
    #[error(transparent)]
    SchemaError(#[from] SchemaError),
}

/// Validates a subgraph's schema by fetching it from the given url and comparing
/// against this repo schema
pub fn validate_subgraph_schema(subgraph_url: &str) -> Result<bool, SchemaValidationError> {
    let main_schema = read_to_string("./schema/orderbook.graphql")?;
    let capabilities = reqwest::blocking::Client::new()
        .post(subgraph_url)
        .run_graphql(CapabilitiesQuery::build(()))?
        .data
        .ok_or(SchemaValidationError::UndefinedSchema)?
        .capabilities();
    let schema = reqwest::blocking::Client::new()
        .post(subgraph_url)
        .run_graphql(IntrospectionQuery::with_capabilities(capabilities))?
        .data
        .ok_or(SchemaValidationError::UndefinedSchema)?
        .into_schema()?
        .to_sdl();

    Ok(main_schema == schema)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_happy_true() {
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-test/0.7/gn";
        let result = validate_subgraph_schema(subgraph_url).unwrap();
        assert!(result)
    }

    #[test]
    fn test_validate_happy_false() {
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.4/gn";
        let result = validate_subgraph_schema(subgraph_url).unwrap();
        assert!(!result)
    }

    #[test]
    fn test_validate_unhappy() {
        let subgraph_url = "https://api.goldsky.com/api/public";
        let result = validate_subgraph_schema(subgraph_url).is_err();
        assert!(result)
    }
}
