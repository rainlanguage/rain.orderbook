use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::order_detail::{
    Bytes, MultiOrderDetailQuery, MultiOrderDetailQueryVariables, OrderFilter,
};

#[test]
fn multi_order_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Bytes("1234".to_string());
    let request_body = MultiOrderDetailQuery::build(MultiOrderDetailQueryVariables {
        filter: OrderFilter { id_in: vec![id] },
    });

    assert_snapshot!(request_body.query);
}
