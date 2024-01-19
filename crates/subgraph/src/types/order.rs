use crate::schema;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct OrderQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrderQueryVariables")]
pub struct OrderQuery {
    #[arguments(id: $id)]
    pub order: Option<Order>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Order {
    pub id: cynic::Id,
    pub owner: Account,
    pub interpreter: Bytes,
    pub interpreter_store: Bytes,
    pub expression_deployer: Bytes,
    pub expression: Bytes,
    pub timestamp: BigInt,
    pub take_orders: Option<Vec<TakeOrderEntity>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct TakeOrderEntity {
    pub id: cynic::Id,
    pub sender: Account,
    pub input: BigInt,
    pub input_display: BigDecimal,
    pub input_token: Erc20,
    pub output: BigInt,
    pub output_display: BigDecimal,
    pub output_token: Erc20,
    #[cynic(rename = "IORatio")]
    pub ioratio: BigDecimal,
    pub timestamp: BigInt,
    pub transaction: Transaction,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Transaction {
    pub block_number: BigInt,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub symbol: String,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: Bytes,
}

#[typeshare]
#[serde(rename = "BigDecimalString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[serde(rename = "BigIntString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[serde(rename = "BytesString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
