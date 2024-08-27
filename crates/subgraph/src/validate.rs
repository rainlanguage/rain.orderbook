use cynic::{GraphQlResponse, QueryBuilder};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SchemaError};
use std::fs::read_to_string;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaValidationError {
    #[error("undefined schema")]
    UndefinedSchema,
    #[error("{0}")]
    ErrorResponse(String),
    #[error(transparent)]
    ReadMainSchemaError(#[from] std::io::Error),
    #[error(transparent)]
    SchemaError(#[from] SchemaError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

/// Validates a subgraph's schema by fetching it from the given url and comparing
/// against this repo schema
pub async fn validate_subgraph_schema(subgraph_url: &str) -> Result<bool, SchemaValidationError> {
    let main_schema = read_to_string("./schema/orderbook.graphql")?;

    let client = reqwest::Client::new();

    let capabilities_response = client
        .post(subgraph_url)
        .json(&CapabilitiesQuery::build(()))
        .send()
        .await?;
    let capabilities = if !capabilities_response.status().is_success() {
        let body_string = capabilities_response
            .text()
            .await
            .map_err(SchemaValidationError::ReqwestError)?;

        Ok::<cynic::GraphQlResponse<CapabilitiesQuery>, serde_json::Error>(serde_json::from_str::<
            GraphQlResponse<CapabilitiesQuery>,
        >(&body_string)?)
    } else {
        Ok(capabilities_response
            .json::<GraphQlResponse<CapabilitiesQuery>>()
            .await?)
    }?
    .data
    .ok_or(SchemaValidationError::UndefinedSchema)?
    .capabilities();

    let schema_response = client
        .post(subgraph_url)
        .json(&IntrospectionQuery::with_capabilities(capabilities))
        .send()
        .await?;
    let schema = if !schema_response.status().is_success() {
        let body_string = schema_response
            .text()
            .await
            .map_err(SchemaValidationError::ReqwestError)?;

        Ok::<cynic::GraphQlResponse<IntrospectionQuery>, serde_json::Error>(serde_json::from_str::<
            GraphQlResponse<IntrospectionQuery>,
        >(&body_string)?)
    } else {
        Ok(schema_response
            .json::<GraphQlResponse<IntrospectionQuery>>()
            .await?)
    }?
    .data
    .ok_or(SchemaValidationError::UndefinedSchema)?
    .into_schema()?
    .to_sdl();

    Ok(main_schema == schema)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_validate_happy_true() {
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-test/0.7/gn";
        let result = validate_subgraph_schema(subgraph_url).await.unwrap();
        assert!(result)
    }

    #[tokio::test]
    async fn test_validate_happy_false() {
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.4/gn";
        let result = validate_subgraph_schema(subgraph_url).await.unwrap();
        assert!(!result)
    }

    #[tokio::test]
    async fn test_validate_unhappy() {
        let subgraph_url = "https://api.goldsky.com/api/public";
        let result = validate_subgraph_schema(subgraph_url).await.is_err();
        assert!(result)
    }
}
