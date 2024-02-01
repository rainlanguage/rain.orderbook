use rain_orderbook_subgraph_client::types::orders_list::{
    OrdersListQuery, OrdersListQueryVariables,
};

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = OrdersListQuery::build(OrdersListQueryVariables {
        skip: Some(0),
        first: Some(10),
    });

    let expected_query = "query OrdersListQuery($first: Int, $skip: Int) {
  orders(orderBy: timestamp, orderDirection: desc, skip: $skip, first: $first) {
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
        id
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
        id
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
