use super::common::{Bytes, RemoveOrderWithOrder};
use crate::schema;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionRemoveOrdersVariables {
    pub id: Bytes,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TransactionRemoveOrdersVariables")]
#[typeshare]
pub struct TransactionRemoveOrdersQuery {
    #[arguments(where: { transaction_: { id: $id } })]
    pub remove_orders: Vec<RemoveOrderWithOrder>,
}
