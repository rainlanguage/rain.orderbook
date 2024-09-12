use insta::assert_snapshot;
use rain_orderbook_subgraph_client::types::common::*;
use rain_orderbook_subgraph_client::types::order::{
    BatchOrderDetailQuery, BatchOrderDetailQueryVariables, OrderIdList,
};

#[test]
fn batch_order_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Bytes("1234".to_string());
    let request_body = BatchOrderDetailQuery::build(BatchOrderDetailQueryVariables {
        id_list: OrderIdList { id_in: vec![id] },
    });

    assert_snapshot!(request_body.query);
}
