use super::common::*;
use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
#[typeshare]
pub struct BatchOrderDetailQueryVariables {
    #[cynic(rename = "id_list")]
    pub id_list: OrderIdList,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Order_filter")]
#[typeshare]
pub struct OrderIdList {
    #[cynic(rename = "id_in")]
    pub id_in: Vec<Bytes>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "BatchOrderDetailQueryVariables")]
#[typeshare]
pub struct BatchOrderDetailQuery {
    #[typeshare(typescript(type = "OrderSubgraph[]"))]
    #[arguments(where: $id_list)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrdersListQueryVariables")]
#[typeshare]
pub struct OrdersListQuery {
    #[typeshare(typescript(type = "OrderSubgraph[]"))]
    #[arguments(orderBy: "timestampAdded", orderDirection: "desc", skip: $skip, first: $first, where: $filters)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct OrderDetailQuery {
    #[arguments(id: $id)]
    #[typeshare(typescript(type = "OrderSubgraph"))]
    pub order: Option<Order>,
}
