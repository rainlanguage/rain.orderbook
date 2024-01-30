use rain_orderbook_subgraph_client::types::vaults_list::{
    VaultsListQuery, VaultsListQueryVariables,
};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = VaultsListQuery::build(VaultsListQueryVariables { skip: Some(0), first: Some(10), });

    let expected_query = "query VaultsListQuery($first: Int, $skip: Int) {
  tokenVaults(orderBy: owner__id, orderDirection: desc, skip: $skip, first: $first) {
    id
    owner {
      id
    }
    vaultId
    token {
      id
      name
      symbol
      decimals
    }
    balanceDisplay
    balance
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
