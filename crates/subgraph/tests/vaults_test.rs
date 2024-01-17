use rain_orderbook_subgraph_queries::types::vaults::VaultsQuery;

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
    token {
      name
      symbol
      decimals
      totalSupply
      totalSupplyDisplay
    }
    balance
    balanceDisplay
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
