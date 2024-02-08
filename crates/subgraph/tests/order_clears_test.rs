use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::order_clears_list::{
    OrderClearsListQuery, OrderClearsListQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = OrderClearsListQuery::build(OrderClearsListQueryVariables {
        skip: Some(0),
        first: Some(10),
    });

    assert_snapshot!(request_body.query);
}
