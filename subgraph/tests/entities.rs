mod generated;
mod subgraph;
mod utils;

use ethers::{
    signers::Signer,
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use subgraph::{wait, Query};
use utils::{
    bytes_to_h256,
    cbor::{decode_rain_meta, encode_rain_docs, RainMapDoc},
    deploy::{deploy_erc20_mock, get_orderbook, read_orderbook_meta, touch_deployer},
    events::{get_add_order_event, get_add_order_events, get_new_expression_event},
    get_wallet,
    json_structs::{NewExpressionJson, OrderJson},
    transactions::{generate_multi_add_order, generate_order_config},
};

use ethers::contract::EthCall;
use ethers::core::abi::AbiType;

use generated::{order_book, AddOrderCall, AddOrderFilter, AddOrderReturn};

#[tokio::main]
// #[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Query the OrderBook entity
    let response = Query::orderbook(&orderbook.address())
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
// #[test]
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
    let response = Query::rain_meta_v1(&ob_meta_hashed.clone())
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
// #[test]
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
        let response = Query::content_meta_v1(&content.hash().as_fixed_bytes().into())
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
// #[test]
async fn order_entity_add_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Connect the orderbook to another wallet
    let wallet_1 = get_wallet(1);
    let orderbook = orderbook.connect(&wallet_1).await;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());

    let tx_add_order = add_order_func.send().await.expect("order not sent");

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await;
    let new_expression_data =
        get_new_expression_event(expression_deployer.clone(), &tx_add_order).await;

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    let order_hash = Bytes::from(add_order_data.order_hash);

    let response = Query::order(&order_hash)
        .await
        .expect("cannot get the query response");

    // Data from the event in tx
    let order_data = add_order_data.order;

    // Expected values
    let transaction_hash = tx_add_order.tx_hash().clone();
    let interpreter: Address = expression_deployer.i_interpreter().call().await?;
    let store: Address = expression_deployer.i_store().call().await?;
    // let rain_doc_hashed = Bytes::from(keccak256(rain_doc));
    let rain_doc_hashed = Bytes::from(keccak256(order_config.meta));
    let order_json_string = OrderJson::from_order(order_data.clone()).to_json_string();
    let expression_json_string =
        NewExpressionJson::from_event(new_expression_data).to_json_string();

    let is_order_exist: bool = orderbook
        .order_exists(bytes_to_h256(&order_hash).into())
        .call()
        .await?;

    // Assertions
    assert_eq!(response.id, order_hash);
    assert_eq!(response.order_hash, order_hash);
    assert_eq!(response.owner, wallet_1.address());

    assert_eq!(response.interpreter, interpreter);
    assert_eq!(response.interpreter_store, store);
    assert_eq!(response.expression_deployer, expression_deployer.address());
    assert_eq!(response.expression, order_data.evaluable.expression);

    assert_eq!(response.order_active, is_order_exist, "wrong order status");
    assert_eq!(response.handle_i_o, order_data.handle_io);
    assert_eq!(response.meta, rain_doc_hashed);
    assert_eq!(response.emitter, wallet_1.address());

    assert_eq!(response.order_json_string, order_json_string);
    assert_eq!(
        response.expression_json_string.unwrap(),
        expression_json_string
    );
    assert_eq!(
        response.transaction,
        Bytes::from(transaction_hash.as_fixed_bytes())
    );

    assert!(
        response.take_orders.is_empty(),
        "take orders not empty at initial addOrder"
    );
    assert!(
        response.orders_clears.is_empty(),
        "order clears not empty at initial addOrder"
    );

    // Iterate over each IO to generate the ID and check if present
    for input in &order_data.valid_inputs {
        let token: Address = input.token;
        let vault_id: U256 = input.vault_id;
        let id = format!("{}-{:?}-{}", order_hash, token, vault_id);

        assert!(response.valid_inputs.contains(&id), "Missing IO in order");
    }

    for output in &order_data.valid_outputs {
        let token: Address = output.token;
        let vault_id: U256 = output.vault_id;
        let id = format!("{}-{:?}-{}", order_hash, token, vault_id);

        assert!(response.valid_outputs.contains(&id), "Missing IO in order");
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn order_entity_remove_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Connect the orderbook to another wallet
    let wallet_1 = get_wallet(1);
    let orderbook = orderbook.connect(&wallet_1).await;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await.expect("order not sent");

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await;

    let order_hash = Bytes::from(add_order_data.order_hash);

    // Data from the event in tx
    let order_data = add_order_data.order;

    // Remove the order
    let remove_order_fnc = orderbook.remove_order(order_data);
    let _ = remove_order_fnc.send().await.expect("order not removed");

    // Current order status
    let is_order_exist: bool = orderbook
        .order_exists(bytes_to_h256(&order_hash).into())
        .call()
        .await?;

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    let response = Query::order(&order_hash)
        .await
        .expect("cannot get the query response");

    assert_eq!(response.order_active, is_order_exist, "wrong order status");

    Ok(())
}

#[tokio::main]
// #[test]
async fn io_entity_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await.expect("order not sent");

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await;

    // Order hash
    let order_hash = Bytes::from(add_order_data.order_hash);
    let order_owner: Address = add_order_data.order.owner;

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Inputs
    for (index, input) in order_config.valid_inputs.iter().enumerate() {
        let token: Address = input.token;
        let vault_id: U256 = input.vault_id;
        let input_id = format!("{}-{:?}-{}", order_hash, token, vault_id);

        let vault_entity_id = format!("{}-{:?}", vault_id, order_owner);
        let token_vault_entity_id = format!("{}-{:?}-{:?}", vault_id, order_owner, token);

        let response = Query::i_o(&input_id)
            .await
            .expect("cannot get the query response");

        assert_eq!(response.id, input_id);
        assert_eq!(response.token, token);
        assert_eq!(response.decimals, 18); // TODO: Make a wrapper around the token address with the ERC20Mock
        assert_eq!(response.vault_id, vault_id);
        assert_eq!(response.order, order_hash);
        assert_eq!(response.index, index as u8);
        assert_eq!(response.vault, vault_entity_id);
        assert_eq!(response.token_vault, token_vault_entity_id);
    }

    // Outputs
    for (index, output) in order_config.valid_outputs.iter().enumerate() {
        let token: Address = output.token;
        let vault_id: U256 = output.vault_id;
        let output_id = format!("{}-{:?}-{}", order_hash, token, vault_id);

        let vault_entity_id = format!("{}-{:?}", vault_id, order_owner);
        let token_vault_entity_id = format!("{}-{:?}-{:?}", vault_id, order_owner, token);

        let response = Query::i_o(&output_id)
            .await
            .expect("cannot get the query response");

        assert_eq!(response.id, output_id);
        assert_eq!(response.token, token);
        assert_eq!(response.decimals, 18); // TODO: Make a wrapper around the token address with the ERC20Mock
        assert_eq!(response.vault_id, vault_id);
        assert_eq!(response.order, order_hash);
        assert_eq!(response.index, index as u8);
        assert_eq!(response.vault, vault_entity_id);
        assert_eq!(response.token_vault, token_vault_entity_id);
    }

    Ok(())
}

#[tokio::main]
#[test]
async fn vault_entity_add_orders_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None)
        .await
        .expect("cannot deploy expression_deployer");

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (C)
    let token_c = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (D)
    let token_d = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Generate TWO order configs with similar vault IDs.
    // All the TokenVaults inside the Vault should be present
    let order_config_a =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;
    let order_config_b =
        generate_order_config(&expression_deployer, &token_c, None, &token_d, None).await;

    // Encode them to send them with multicall
    let multi_orders = generate_multi_add_order(vec![&order_config_a, &order_config_b]);

    // Add the orders with multicall
    let multicall_func = orderbook.multicall(multi_orders);
    let tx_multicall = multicall_func.send().await.expect("multicall not sent");

    // Get all orders events (logs) from the transaction
    let order_events = get_add_order_events(orderbook, &tx_multicall).await;

    println!("order_events: \n {:?}\n", order_events);

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

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
