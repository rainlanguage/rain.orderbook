use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::vault_detail::{
    VaultDetailQuery, VaultDetailQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = VaultDetailQuery::build(VaultDetailQueryVariables { id });

    assert_snapshot!(request_body.query);
}
