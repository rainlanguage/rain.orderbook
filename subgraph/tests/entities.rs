mod generated;
mod subgraph;
mod utils;

use ethers::{
    signers::Signer,
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use generated::{EvaluableConfigV2, Io, OrderConfigV2};
use subgraph::{wait, Query};
use utils::{
    cbor::{decode_rain_meta, encode_rain_docs, RainMapDoc},
    deploy::{
        deploy_erc20_mock, get_orderbook, ob_connect_to, read_orderbook_meta, touch_deployer,
    },
    events::{get_add_order_event, get_new_expression_event},
    get_wallet,
    json_structs::NewExpressionJson,
    mock_rain_doc,
};

#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Query the OrderBook entity
    let response = Query::orderbook(orderbook.address())
        .await
        .expect("cannot get the ob query response");

    // This wallet is used to deploy the OrderBook at initialization, so it is the deployer
    let wallet_0 = get_wallet(0);

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta_hashed = Bytes::from(keccak256(read_orderbook_meta()));

    assert_eq!(response.id, orderbook.address());
    assert_eq!(response.address, orderbook.address());
    assert_eq!(response.deployer, wallet_0.address());
    assert_eq!(response.meta, ob_meta_hashed);

    Ok(())
}

#[tokio::main]
#[test]
async fn rain_meta_v1_entity_test() -> anyhow::Result<()> {
    // Always checking if OB is deployed, so we attemp to obtaing it
    let _ = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta = read_orderbook_meta();
    let ob_meta_bytes = Bytes::from(ob_meta.clone());
    let ob_meta_hashed = Bytes::from(keccak256(ob_meta.clone()));
    let ob_meta_decoded = decode_rain_meta(ob_meta.clone().into())?;

    // Query the RainMetaV1 entity
    let response = Query::rain_meta_v1(ob_meta_hashed.clone())
        .await
        .expect("cannot get the rain meta query response");

    assert_eq!(response.id, ob_meta_hashed);
    assert_eq!(response.meta_bytes, ob_meta_bytes);

    for content in ob_meta_decoded {
        let content_id: Bytes = content.hash().to_fixed_bytes().into();
        assert!(
            response.content.contains(&content_id),
            "Missing id '{}' in decoded contents: {:?}",
            content_id,
            response.content
        );
    }

    Ok(())
}

#[tokio::main]
#[test]
async fn content_meta_v1_entity_test() -> anyhow::Result<()> {
    // Always checking if OB is deployed, so we attemp to obtaing it
    let _ = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta = read_orderbook_meta();
    let ob_meta_hashed = Bytes::from(keccak256(ob_meta.clone()));
    let ob_meta_decoded = decode_rain_meta(ob_meta.clone().into())?;

    for content in ob_meta_decoded {
        // Query the ContentMetaV1 entity
        let response = Query::content_meta_v1(content.hash().as_fixed_bytes().into())
            .await
            .expect("cannot get the query response");

        // Make the asserts
        assert_eq!(response.id, content.hash().as_bytes().to_vec());
        assert_eq!(response.raw_bytes, content.encode());
        assert_eq!(response.magic_number, content.magic_number);
        assert_eq!(response.payload, content.payload);

        assert_eq!(response.content_type, content.content_type);
        assert_eq!(response.content_encoding, content.content_encoding);
        assert_eq!(response.content_language, content.content_language);

        assert!(
            response.parents.contains(&ob_meta_hashed),
            "Missing parent id '{}' in {:?}",
            ob_meta_hashed,
            response.parents
        );
    }

    Ok(())
}

#[tokio::main]
#[test]
async fn order_entity_add_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Connect the orderbook to another wallet
    let wallet_1 = get_wallet(1);
    let orderbook = ob_connect_to(orderbook, &wallet_1).await;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token A");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token B");

    // * Build OrderConfig
    // Build IO (input)
    let io_input = Io {
        token: token_a.address(),
        decimals: token_a.decimals().await.unwrap(),
        vault_id: U256::from(0),
    };

    // Build IO (output)
    let io_output = Io {
        token: token_b.address(),
        decimals: token_b.decimals().await.unwrap(),
        vault_id: U256::from(0),
    };

    let data_parse = Bytes::from_static(b"_ _ _:block-timestamp() chain-id() block-number();:;");
    let (bytecode, constants) = expression_deployer
        .parse(data_parse.clone())
        .await
        .expect("cannot get value from parse");

    let rain_doc = mock_rain_doc();

    // Build EvaluableConfigV2
    let eval_config = EvaluableConfigV2 {
        deployer: expression_deployer.address(),
        bytecode,
        constants,
    };

    let config = OrderConfigV2 {
        valid_inputs: vec![io_input],
        valid_outputs: vec![io_output],
        evaluable_config: eval_config,
        meta: rain_doc.clone(),
    };

    // Add the order
    let add_order_func = orderbook.add_order(config);
    let tx_add_order = add_order_func.send().await.expect("order not sent");

    let add_order_data = get_add_order_event(orderbook.clone(), &tx_add_order).await;
    println!("add_order_data: {:?}\n", add_order_data);

    let new_expression_data =
        get_new_expression_event(expression_deployer.clone(), tx_add_order).await;
    println!("new_expression_data: {:?}\n", new_expression_data);

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    let order_id = Bytes::from(add_order_data.order_hash);

    let response = Query::order(order_id)
        .await
        .expect("cannot get the query response");

    println!("Response: {:?}\n", response);

    // Data from the event in tx
    let order_data = add_order_data.order;

    // Expected values
    let interpreter: Address = expression_deployer.i_interpreter().call().await?;
    let store: Address = expression_deployer.i_store().call().await?;
    let rain_doc_hashed = Bytes::from(keccak256(rain_doc));
    let expression_json_string =
        NewExpressionJson::from_event(new_expression_data).to_json_string();

    assert_eq!(response.id, Bytes::from(&add_order_data.order_hash));
    assert_eq!(response.order_hash, Bytes::from(&add_order_data.order_hash));
    assert_eq!(response.owner, wallet_1.address());

    assert_eq!(response.interpreter, interpreter);
    assert_eq!(response.interpreter_store, store);
    assert_eq!(response.expression_deployer, expression_deployer.address());
    assert_eq!(response.expression, order_data.evaluable.expression);

    assert_eq!(response.order_active, true, "order not active");
    assert_eq!(response.handle_i_o, order_data.handle_io);
    assert_eq!(response.meta, rain_doc_hashed);

    assert_eq!(
        response.expression_json_string.unwrap(),
        expression_json_string
    );

    // "validInputs
    // validInputs: [IO!]

    // "validOutputs"
    // validOutputs: [IO!]

    // OrderJSON could be parsed and used to send other transaction
    // orderJSONString: String!

    // orderJSONString: String!
    // expressionJSONString: String
    // "Timestamp when the order was added"
    // transaction: Transaction!
    // emitter: Account!
    // timestamp: BigInt!
    // "Take Order entities that use this order"
    // takeOrders: [TakeOrderEntity!] @derivedFrom(field: "order")
    // "Order Clear entities that use this order"
    // ordersClears: [OrderClear!]

    Ok(())
}

#[test]
fn util_cbor_meta_test() -> anyhow::Result<()> {
    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta: Vec<u8> = read_orderbook_meta();

    let output: Vec<RainMapDoc> = decode_rain_meta(ob_meta.clone().into())?;

    let encoded_again = encode_rain_docs(output);

    assert_eq!(ob_meta, encoded_again);

    Ok(())
}
