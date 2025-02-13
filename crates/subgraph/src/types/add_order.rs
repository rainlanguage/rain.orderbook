use super::common::AddOrder;
use crate::schema;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct GetTransactionAddOrdersVariables<'a> {
    pub id: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetTransactionAddOrdersVariables")]
pub struct GetTransactionAddOrders {
    #[arguments(where: { transaction_: { id: $id } })]
    pub add_orders: Vec<AddOrder>,
}

#[derive(cynic::QueryFragment, Debug)]
#[typeshare]
pub struct Order {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct Bytes(pub String);
