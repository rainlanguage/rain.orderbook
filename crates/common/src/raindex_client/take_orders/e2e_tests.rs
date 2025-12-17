use crate::raindex_client::take_orders::TakeOrdersRequest;
use crate::raindex_client::tests::get_test_yaml;
use crate::raindex_client::RaindexClient;
use crate::raindex_client::RaindexError;
use crate::take_orders::MinReceiveMode;
use crate::test_helpers::dotrain::create_dotrain_config_with_params;
use crate::test_helpers::local_evm::{
    create_vault, fund_and_approve_taker, fund_standard_two_token_vault,
    setup_test as base_setup_test,
};
use crate::test_helpers::orders::deploy::deploy_order;
use crate::test_helpers::subgraph::{create_sg_order_json, get_minimal_yaml_for_chain};
use alloy::primitives::{B256, U256};
use alloy::sol_types::SolCall;
use httpmock::MockServer;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use serde_json::json;

fn high_price_cap() -> String {
    "1000000".to_string()
}

#[tokio::test]
async fn test_get_take_orders_calldata_no_orders_returns_no_liquidity() {
    let sg_server = MockServer::start_async().await;

    sg_server.mock(|when, then| {
        when.path("/sg1");
        then.status(200).json_body_obj(&json!({
            "data": {
                "orders": []
            }
        }));
    });
    sg_server.mock(|when, then| {
        when.path("/sg2");
        then.status(200).json_body_obj(&json!({
            "data": {
                "orders": []
            }
        }));
    });

    let client = RaindexClient::new(
        vec![get_test_yaml(
            &sg_server.url("/sg1"),
            &sg_server.url("/sg2"),
            "http://localhost:0/unused_rpc1",
            "http://localhost:0/unused_rpc2",
        )],
        None,
    )
    .unwrap();

    let res = client
        .get_take_orders_calldata(TakeOrdersRequest {
            taker: "0x1111111111111111111111111111111111111111".to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            buy_amount: "1".to_string(),
            price_cap: high_price_cap(),
            min_receive_mode: MinReceiveMode::Partial,
        })
        .await;

    assert!(
        matches!(res, Err(RaindexError::NoLiquidity)),
        "Expected NoLiquidity error when subgraph returns empty orders, got: {:?}",
        res
    );
}

#[tokio::test]
async fn test_get_take_orders_calldata_no_candidates_returns_no_liquidity() {
    let setup = base_setup_test().await;
    let sg_server = MockServer::start_async().await;

    let vault_id = B256::from(U256::from(1u64));
    let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
    let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

    let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
    let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

    let order_json = create_sg_order_json(
        &setup,
        &order_bytes,
        order_hash,
        vec![vault1.clone(), vault2.clone()],
        vec![vault1.clone(), vault2.clone()],
    );

    sg_server.mock(|when, then| {
        when.path("/sg");
        then.status(200).json_body_obj(&json!({
            "data": {
                "orders": [order_json]
            }
        }));
    });

    let yaml = get_minimal_yaml_for_chain(
        123,
        &setup.local_evm.url().to_string(),
        &sg_server.url("/sg"),
        &setup.orderbook.to_string(),
    );

    let client = RaindexClient::new(vec![yaml], None).unwrap();

    let res = client
        .get_take_orders_calldata(TakeOrdersRequest {
            taker: setup.owner.to_string(),
            chain_id: 123,
            sell_token: setup.token1.to_string(),
            buy_token: setup.token2.to_string(),
            buy_amount: "10".to_string(),
            price_cap: high_price_cap(),
            min_receive_mode: MinReceiveMode::Partial,
        })
        .await;

    assert!(
        matches!(res, Err(RaindexError::NoLiquidity)),
        "Expected NoLiquidity error when no candidates (no vault balance), got: {:?}",
        res
    );
}

#[tokio::test]
async fn test_get_take_orders_calldata_happy_path_returns_valid_config() {
    let setup = base_setup_test().await;
    let sg_server = MockServer::start_async().await;

    let vault_id = B256::from(U256::from(1u64));
    fund_standard_two_token_vault(&setup, vault_id).await;

    let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
    let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

    let dotrain = create_dotrain_config_with_params(&setup, "100", "2");
    let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

    let order_json = create_sg_order_json(
        &setup,
        &order_bytes,
        order_hash,
        vec![vault1.clone(), vault2.clone()],
        vec![vault1.clone(), vault2.clone()],
    );

    sg_server.mock(|when, then| {
        when.path("/sg");
        then.status(200).json_body_obj(&json!({
            "data": {
                "orders": [order_json]
            }
        }));
    });

    let yaml = get_minimal_yaml_for_chain(
        123,
        &setup.local_evm.url().to_string(),
        &sg_server.url("/sg"),
        &setup.orderbook.to_string(),
    );

    let client = RaindexClient::new(vec![yaml], None).unwrap();

    let taker = setup.local_evm.signer_wallets[1].default_signer().address();
    fund_and_approve_taker(
        &setup,
        setup.token1,
        taker,
        setup.orderbook,
        U256::from(10).pow(U256::from(24)),
    )
    .await;

    let result = client
        .get_take_orders_calldata(TakeOrdersRequest {
            taker: taker.to_string(),
            chain_id: 123,
            sell_token: setup.token1.to_string(),
            buy_token: setup.token2.to_string(),
            buy_amount: "100".to_string(),
            price_cap: "10".to_string(),
            min_receive_mode: MinReceiveMode::Partial,
        })
        .await
        .expect("Should succeed with funded vault and valid order");

    assert_eq!(
        result.orderbook, setup.orderbook,
        "Orderbook address should match"
    );

    let decoded = takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
    let config = decoded.config;

    assert!(
        !config.orders.is_empty(),
        "Should have at least one order in config"
    );

    assert_eq!(
        config.minimumInput,
        Float::zero().unwrap().get_inner(),
        "minimumInput should be zero for Partial mode"
    );

    assert!(
        !result.prices.is_empty(),
        "Should have at least one price in result"
    );

    let expected_ratio = Float::parse("2".to_string()).unwrap();
    assert!(
        result.prices[0].eq(expected_ratio).unwrap(),
        "Price should match expected ratio of 2, got: {:?}",
        result.prices[0].format()
    );

    let zero = Float::zero().unwrap();
    assert!(
        result.effective_price.gt(zero).unwrap(),
        "Effective price should be > 0"
    );
}

#[tokio::test]
async fn test_get_take_orders_calldata_invalid_address_returns_from_hex_error() {
    let client = RaindexClient::new(
        vec![get_test_yaml(
            "http://localhost:0/unused_sg1",
            "http://localhost:0/unused_sg2",
            "http://localhost:0/unused_rpc1",
            "http://localhost:0/unused_rpc2",
        )],
        None,
    )
    .unwrap();

    let res = client
        .get_take_orders_calldata(TakeOrdersRequest {
            taker: "0x1111111111111111111111111111111111111111".to_string(),
            chain_id: 1,
            sell_token: "not-an-address".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            buy_amount: "1".to_string(),
            price_cap: high_price_cap(),
            min_receive_mode: MinReceiveMode::Partial,
        })
        .await;

    assert!(
        matches!(res, Err(RaindexError::FromHexError(_))),
        "Expected FromHexError for invalid sellToken address, got: {:?}",
        res
    );
}

#[tokio::test]
async fn test_get_take_orders_calldata_invalid_float_returns_float_error() {
    let client = RaindexClient::new(
        vec![get_test_yaml(
            "http://localhost:0/unused_sg1",
            "http://localhost:0/unused_sg2",
            "http://localhost:0/unused_rpc1",
            "http://localhost:0/unused_rpc2",
        )],
        None,
    )
    .unwrap();

    let res = client
        .get_take_orders_calldata(TakeOrdersRequest {
            taker: "0x1111111111111111111111111111111111111111".to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            buy_amount: "not-a-float".to_string(),
            price_cap: high_price_cap(),
            min_receive_mode: MinReceiveMode::Partial,
        })
        .await;

    assert!(
        matches!(res, Err(RaindexError::Float(_))),
        "Expected Float error for invalid buyAmount, got: {:?}",
        res
    );
}
