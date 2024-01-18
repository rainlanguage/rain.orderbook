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
    interpreter
    interpreterStore
    expressionDeployer
    expression
    timestamp
    takeOrders {
      id
      sender {
        id
      }
      input
      inputDisplay
      inputToken {
        symbol
      }
      output
      outputDisplay
      outputToken {
        symbol
      }
      IORatio
      timestamp
      transaction {
        blockNumber
      }
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
