use super::common::*;
use crate::schema;
use typeshare::typeshare;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct TransactionDetailQuery {
    #[arguments(id: $id)]
    #[typeshare(typescript(type = "TransactionSubgraph"))]
    pub transaction: Option<Transaction>,
}
