use rain_orderbook_subgraph_queries::types::orders::{OrdersQuery, OrdersQueryVariables};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let operation = OrdersQuery::build(OrdersQueryVariables { active: None });

    insta::assert_snapshot!(operation.query);
}
