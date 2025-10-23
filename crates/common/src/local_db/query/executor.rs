use async_trait::async_trait;

use super::{FromDbJson, LocalDbQueryError};

/// Backend-neutral executor for running SQL against the local DB backend.
///
/// Implementations provide text and JSON query methods that map backend
/// errors into `LocalDbQueryError` and deserialize JSON into target types
/// via the `FromDbJson` bound.
#[async_trait(?Send)]
pub trait LocalDbQueryExecutor {
    async fn query_json<T>(&self, sql: &str) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson;

    async fn query_text(&self, sql: &str) -> Result<String, LocalDbQueryError>;
}
