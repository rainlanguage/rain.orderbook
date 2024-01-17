use crate::types::{
    orders::{Order, OrdersQuery},
    vaults::{TokenVault, VaultsQuery},
};
use anyhow::{anyhow, Result};
use cynic::{
    serde::{Deserialize, Serialize},
    GraphQlResponse, QueryBuilder, QueryFragment,
};
use reqwest::Url;

pub struct OrderbookSubgraphClient {
    url: Url,
}

impl OrderbookSubgraphClient {
    pub async fn new(url: Url) -> Self {
        Self { url }
    }

    async fn _query<R: QueryFragment + QueryBuilder<V> + for<'a> Deserialize<'a>, V: Serialize>(
        &self,
        variables: V,
    ) -> Result<R> {
        let request_body = R::build(variables);

        let response = reqwest::Client::new()
            .post(self.url.clone())
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

    pub async fn orders(&self) -> Result<Vec<Order>> {
        let data = self._query::<OrdersQuery, ()>(()).await?;

        Ok(data.orders)
    }

    pub async fn vaults(&self) -> Result<Vec<TokenVault>> {
        let data = self._query::<VaultsQuery, ()>(()).await?;

        Ok(data.token_vaults)
    }
}
