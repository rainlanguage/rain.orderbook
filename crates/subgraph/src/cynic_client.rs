use anyhow::{anyhow, Result};
use cynic::{
    serde::{Deserialize, Serialize},
    GraphQlResponse, QueryBuilder, QueryFragment,
};
use reqwest::Url;

pub trait CynicClient {
    async fn query<R: QueryFragment + QueryBuilder<V> + for<'a> Deserialize<'a>, V: Serialize>(
        &self,
        base_url: Url,
        variables: V,
    ) -> Result<R> {
        let request_body = R::build(variables);

        let response = reqwest::Client::new()
            .post(base_url.clone())
            .json(&request_body)
            .send()
            .await?;

        let response_deserialized: GraphQlResponse<R> =
            response.json::<GraphQlResponse<R>>().await?;

        match response_deserialized.errors {
            Some(errors) => Err(anyhow!(
                "Graphql: {}",
                errors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
            None => response_deserialized
                .data
                .ok_or(anyhow!("Subgraph query returned no data")),
        }
    }
}
