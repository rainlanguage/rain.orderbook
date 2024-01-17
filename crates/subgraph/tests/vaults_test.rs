use rain_orderbook_subgraph_queries::types::vaults::VaultsQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = VaultsQuery::build(());

    let expected_query = "query VaultsQuery {
  vaults(orderBy: owner__id, orderDirection: desc) {
    id
    owner {
      id
    }
    tokenVaults {
      id
      token {
        name
        symbol
        decimals
        totalSupply
        totalSupplyDisplay
      }  
      balanceDisplay
        balance
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
