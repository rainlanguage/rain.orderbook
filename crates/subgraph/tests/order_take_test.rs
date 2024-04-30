use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::order_take_detail::{
    OrderTakeDetailQuery, OrderTakeDetailQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderTakeDetailQuery::build(OrderTakeDetailQueryVariables { id: &id });

    assert_snapshot!(request_body.query);
}
