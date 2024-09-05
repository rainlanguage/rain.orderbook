use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::common::*;
use rain_orderbook_subgraph_client::types::order::OrdersListQuery;

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = OrdersListQuery::build(PaginationQueryVariables {
        skip: Some(0),
        first: Some(10),
    });

    assert_snapshot!(request_body.query);
}
