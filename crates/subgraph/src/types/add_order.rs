use super::common::{AddOrderWithOrder, Bytes};
use crate::schema;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionAddOrdersVariables {
    pub id: Bytes,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TransactionAddOrdersVariables")]
#[typeshare]
pub struct TransactionAddOrdersQuery {
    #[arguments(where: { transaction_: { id: $id } })]
    pub add_orders: Vec<AddOrderWithOrder>,
}
