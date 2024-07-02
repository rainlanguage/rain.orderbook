use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::order_detail::{
    OrderDetailQuery, OrderDetailQueryVariables,
};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderDetailQuery::build(OrderDetailQueryVariables { id });

    assert_snapshot!(request_body.query);
}
