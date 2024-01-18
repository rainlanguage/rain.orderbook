use rain_orderbook_subgraph_queries::types::orders::OrdersQuery;

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = OrdersQuery::build(());

    let expected_query = "query OrdersQuery {
  orders(orderBy: timestamp, orderDirection: desc) {
    id
    timestamp
    handleIO
    orderJSONString
    owner {
      id
    }
    orderActive
    expression
    interpreter
    interpreterStore
    transaction {
      id
    }
    validInputs {
      token {
        id
        symbol
        decimals
      }
      tokenVault {
        balance
      }
      vault {
        id
      }
    }
    validOutputs {
      token {
        id
        symbol
        decimals
      }
      tokenVault {
        balance
      }
      vault {
        id
      }
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
