use super::common::{AddOrder, Bytes};
use crate::schema;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionAddOrdersVariables<'a> {
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TransactionAddOrdersVariables")]
#[typeshare]
pub struct TransactionAddOrdersQuery {
    #[arguments(where: { transaction_: { id: $id } })]
    pub add_orders: Vec<AddOrder>,
}

#[derive(cynic::QueryFragment, Debug)]
#[typeshare]
pub struct Order {
    pub id: Bytes,
}
