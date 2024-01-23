use cynic::Id;
use rain_orderbook_subgraph_queries::types::vault::{VaultQuery, VaultQueryVariables};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = VaultQuery::build(VaultQueryVariables { id: &id });

    let expected_query = "query VaultQuery($id: ID!) {
  tokenVault(id: $id) {
    id
    owner {
      id
    }
    balance
    balanceDisplay
    token {
      name
      symbol
      decimals
    }
    vault {
      deposits(where: {tokenVault_: {id: $id, }, }) {
        id
        transaction {
          id
        }
        timestamp
        sender {
          id
        }
        amount
        amountDisplay
      }
      withdraws(where: {tokenVault_: {id: $id, }, }) {
        id
        sender {
          id
        }
        transaction {
          id
        }
        timestamp
        amount
        amountDisplay
        requestedAmount
        requestedAmountDisplay
        tokenVault {
          balanceDisplay
        }
      }
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
