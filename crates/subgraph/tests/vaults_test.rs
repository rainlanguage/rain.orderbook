use rain_orderbook_subgraph_client::types::vaults::VaultsQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = VaultsQuery::build(());

    let expected_query = "query VaultsQuery {
  tokenVaults(orderBy: owner__id, orderDirection: desc) {
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
