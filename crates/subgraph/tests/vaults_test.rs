use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::vaults_list::{
    VaultsListQuery, VaultsListQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = VaultsListQuery::build(VaultsListQueryVariables {
        skip: Some(0),
        first: Some(10),
    });

    assert_snapshot!(request_body.query);
}
