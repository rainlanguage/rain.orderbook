use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct OrderTakeDetailQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrderTakeDetailQueryVariables")]
pub struct OrderTakeDetailQuery {
    #[arguments(id: $id)]
    pub take_order_entity: Option<TakeOrderEntity>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct TakeOrderEntity {
    pub id: cynic::Id,
    pub transaction: Transaction,
    pub sender: Account,
    pub timestamp: BigInt,
    pub order: Order,
    #[cynic(rename = "IORatio")]
    pub ioratio: BigDecimal,
    pub input: BigInt,
    pub input_display: BigDecimal,
    pub input_token: Erc20,
    #[cynic(rename = "inputIOIndex")]
    pub input_ioindex: BigInt,
    pub output: BigInt,
    pub output_display: BigDecimal,
    pub output_token: Erc20,
    #[cynic(rename = "outputIOIndex")]
    pub output_ioindex: BigInt,
    pub context: Option<ContextEntity>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Transaction {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Order {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct ContextEntity {
    pub calling_context: Option<Vec<BigInt>>,
    pub calculations_context: Option<Vec<BigInt>>,
    pub vault_inputs_context: Option<Vec<BigInt>>,
    pub vault_outputs_context: Option<Vec<BigInt>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
