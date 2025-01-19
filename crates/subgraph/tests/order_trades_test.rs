use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::common::*;
use rain_orderbook_subgraph_client::types::order_trade::OrderTradesListQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderTradesListQuery::build(PaginationWithTimestampQueryVariables {
        id: Bytes(id.inner().to_string()),
        skip: Some(0),
        first: Some(10),
        timestamp_gte: None,
        timestamp_lte: None,
    });

    assert_snapshot!(request_body.query);
}
