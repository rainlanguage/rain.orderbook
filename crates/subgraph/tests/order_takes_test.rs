use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::order_takes_list::{
    Bytes, OrderTakesListQuery, OrderTakesListQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderTakesListQuery::build(OrderTakesListQueryVariables {
        id: Bytes(id.inner().to_string()),
        skip: Some(0),
        first: Some(10),
    });

    assert_snapshot!(request_body.query);
}
