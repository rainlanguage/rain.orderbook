use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct OrderQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "OrderQueryVariables")]
pub struct OrderQuery {
    #[arguments(id: $id)]
    pub order: Option<Order>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Order {
    pub id: cynic::Id,
    pub owner: Account,
    pub order_active: bool,
    pub interpreter: Bytes,
    pub interpreter_store: Bytes,
    pub expression_deployer: Bytes,
    pub expression: Bytes,
    pub timestamp: BigInt,
    #[cynic(rename = "handleIO")]
    pub handle_io: bool,
    pub valid_inputs: Option<Vec<Io>>,
    pub valid_outputs: Option<Vec<Io>>,
    pub meta: Option<RainMetaV1>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct RainMetaV1 {
    pub meta_bytes: Bytes,
    pub content: Vec<ContentMetaV1>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "IO")]
pub struct Io {
    pub token_vault: TokenVault,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub vault_id: BigInt,
    pub token: Erc20,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ContentMetaV1 {
    pub id: Bytes,
    pub payload: Bytes,
    pub magic_number: BigInt,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub content_language: Option<String>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
