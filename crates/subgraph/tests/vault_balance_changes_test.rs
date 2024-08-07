use cynic::Id;
use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::vault_balance_changes_list::{
    Bytes, VaultBalanceChangesListQuery, VaultBalanceChangesListQueryVariables,
};

#[test]
fn vault_balance_changes_list_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = VaultBalanceChangesListQuery::build(VaultBalanceChangesListQueryVariables {
        id: Bytes(id.inner().to_string()),
        skip: None,
        first: None,
    });

    assert_snapshot!(request_body.query);
}
