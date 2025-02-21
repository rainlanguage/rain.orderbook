use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::common::*;
use rain_orderbook_subgraph_client::types::vault::SgVaultsListQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = SgVaultsListQuery::build(SgVaultsListQueryVariables {
        skip: Some(0),
        first: Some(10),
        filters: None,
    });

    assert_snapshot!(request_body.query);
}
