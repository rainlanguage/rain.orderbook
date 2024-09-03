use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::orders_list::{
    OrdersListQuery, OrdersListQueryVariables,
};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = OrdersListQuery::build(OrdersListQueryVariables {
        skip: Some(0),
        first: Some(10),
        owners: None,
    });

    assert_snapshot!(request_body.query);
}
