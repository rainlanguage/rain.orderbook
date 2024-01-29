use cynic::Id;
use rain_orderbook_subgraph_queries::types::order::{OrderQuery, OrderQueryVariables};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = OrderQuery::build(OrderQueryVariables { id: &id });

    let expected_query = "query OrderQuery($id: ID!) {
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
    validInputs {
      token {
        id
        name
        symbol
        decimals
      }
    }
    validOutputs {
      token {
        id
        name
        symbol
        decimals
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
