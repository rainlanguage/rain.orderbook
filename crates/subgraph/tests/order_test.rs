use cynic::Id;
use rain_orderbook_subgraph_client::types::order_detail::{
    OrderDetailQuery, OrderDetailQueryVariables,
};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderDetailQuery::build(OrderDetailQueryVariables { id: &id });

    let expected_query = "query OrderDetailQuery($id: ID!) {
  order(id: $id) {
    id
    owner {
      id
    }
    orderActive
    interpreter
    interpreterStore
    expressionDeployer
    expression
    timestamp
    handleIO
    validInputs {
      tokenVault {
        id
        vaultId
        token {
          id
          name
          symbol
          decimals
        }
      }
    }
    validOutputs {
      tokenVault {
        id
        vaultId
        token {
          id
          name
          symbol
          decimals
        }
      }
    }
    meta {
      metaBytes
      content {
        id
        payload
        magicNumber
        contentType
        contentEncoding
        contentLanguage
      }
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
