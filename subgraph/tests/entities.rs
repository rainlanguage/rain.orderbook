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
    events::{
        get_add_order_event, get_add_order_events, get_after_clear_event, get_clear_event,
        get_new_expression_event,
    },
    gen_abigen::_abigen_rust_generation,
    generate_random_u256, get_wallet,
    json_structs::{NewExpressionJson, OrderJson},
    numbers::get_amount_tokens,
    transactions::{
        approve_tokens, generate_clear_config, generate_multi_add_order, generate_multi_deposit,
        generate_order_config, mint_tokens, TestDepositConfig,
    },
};

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
async fn order_entity_add_order_test() -> anyhow::Result<()> {
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
// #[test]
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

    // Generate TWO order configs with identical vault ID.
    // All the TokenVaults with same VaultId should be present in the Vault
    let vault_id = generate_random_u256();

    let order_config_a = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;
    let order_config_b = generate_order_config(
        &expression_deployer,
        &token_c,
        Some(vault_id),
        &token_d,
        Some(vault_id),
    )
    .await;

    // Encode them to send them with multicall
    let multi_orders = generate_multi_add_order(vec![&order_config_a, &order_config_b]);

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let wallet_owner = get_wallet(2);
    let orderbook = orderbook.connect(&wallet_owner).await;

    // Add the orders with multicall
    let multicall_func = orderbook.multicall(multi_orders);
    let tx_multicall = multicall_func.send().await.expect("multicall not sent");
    let _ = tx_multicall.await.expect("failed to wait receipt");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    let vault_entity_id = format!("{}-{:?}", vault_id, wallet_owner.address());

    let response = Query::vault(&vault_entity_id)
        .await
        .expect("cannot get the query response");

    // Generate the expetect Token Vault IDs
    let token_vault_a = format!(
        "{}-{:?}-{:?}",
        vault_id,
        wallet_owner.address(),
        token_a.address()
    );
    let token_vault_b = format!(
        "{}-{:?}-{:?}",
        vault_id,
        wallet_owner.address(),
        token_b.address()
    );
    let token_vault_c = format!(
        "{}-{:?}-{:?}",
        vault_id,
        wallet_owner.address(),
        token_c.address()
    );
    let token_vault_d = format!(
        "{}-{:?}-{:?}",
        vault_id,
        wallet_owner.address(),
        token_d.address()
    );

    assert_eq!(response.id, vault_entity_id);
    assert_eq!(response.vault_id, vault_id);
    assert_eq!(response.owner, wallet_owner.address());

    assert!(
        response.token_vaults.contains(&token_vault_a),
        "Missing tokenVault id"
    );
    assert!(
        response.token_vaults.contains(&token_vault_b),
        "Missing tokenVault id"
    );
    assert!(
        response.token_vaults.contains(&token_vault_c),
        "Missing tokenVault id"
    );
    assert!(
        response.token_vaults.contains(&token_vault_d),
        "Missing tokenVault id"
    );

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_entity_deposit_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None)
        .await
        .expect("failed on deploy erc20 token");

    let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens (A and B)
    mint_tokens(&amount, &alice.address(), &token_a)
        .await
        .expect("cannot mint tokens");

    mint_tokens(&amount, &alice.address(), &token_b)
        .await
        .expect("cannot mint tokens");

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await
    .expect("cannot approve tokens");

    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await
    .expect("cannot approve tokens");

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Fill struct with same vaultId in the deposit configurations
    let deposits_config = vec![
        // Config A
        TestDepositConfig {
            token: token_a.address(),
            vault_id: vault_id,
            amount,
        },
        // Config B
        TestDepositConfig {
            token: token_b.address(),
            vault_id: vault_id,
            amount,
        },
    ];

    let multi_deposit = generate_multi_deposit(&deposits_config);

    // Send the deposits with multicall
    let multicall_func = orderbook.multicall(multi_deposit);
    let tx_multicall = multicall_func.send().await.expect("multicall not sent");
    let tx_receipt = tx_multicall.await.expect("failed to wait receipt").unwrap();

    let deposit_tx_hash = &tx_receipt.transaction_hash;

    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Generate the expetect Token Vault IDs
    let token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());
    let token_vault_b = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_b.address());

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    let resp = Query::vault(&vault_entity_id)
        .await
        .expect("cannot get the query response");

    // The whole entity should be created normally
    assert_eq!(resp.id, vault_entity_id);
    assert_eq!(resp.vault_id, vault_id);
    assert_eq!(resp.owner, alice.address());
    assert!(
        resp.token_vaults.contains(&token_vault_a),
        "Missing tokenVault id"
    );
    assert!(
        resp.token_vaults.contains(&token_vault_b),
        "Missing tokenVault id"
    );

    // Should include the deposits made
    for index in 0..deposits_config.len() {
        let deposit_id = format!("{:?}-{}", deposit_tx_hash, index);

        assert!(resp.deposits.contains(&deposit_id), "missing deposit id");
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_entity_add_order_and_deposit_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // The expected vault entity SG ID
    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

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

    // Build OrderConfig with the vaultId
    let order_config = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let _ = add_order_func
        .send()
        .await
        .expect("order not sent")
        .await
        .expect("cannot wait receipt");

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // First query when adding order
    let resp = Query::vault(&vault_entity_id)
        .await
        .expect("cannot get the query response");

    // The whole entity should be created normally when adding the order
    assert_eq!(resp.id, vault_entity_id);

    // Now, make the deposits with a given amount
    let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens (A and B)
    mint_tokens(&amount, &alice.address(), &token_a)
        .await
        .expect("cannot mint tokens");

    mint_tokens(&amount, &alice.address(), &token_b)
        .await
        .expect("cannot mint tokens");

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await
    .expect("cannot approve tokens");

    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await
    .expect("cannot approve tokens");

    // Fill struct with same vaultId in the deposit configurations
    let deposits_config = vec![
        // Config A
        TestDepositConfig {
            token: token_a.address(),
            vault_id: vault_id,
            amount,
        },
        // Config B
        TestDepositConfig {
            token: token_b.address(),
            vault_id: vault_id,
            amount,
        },
    ];
    // The multi deposit data bytes
    let multi_deposit = generate_multi_deposit(&deposits_config);

    // Send the deposits with multicall
    let multicall_func = orderbook.multicall(multi_deposit);
    let tx_multicall = multicall_func.send().await.expect("multicall not sent");
    let tx_receipt = tx_multicall.await.expect("failed to wait receipt").unwrap();

    let deposit_tx_hash = &tx_receipt.transaction_hash;

    // Wait for Subgraph sync
    wait().await.expect("cannot get SG sync status");

    // Second query, using same vault entity ID.
    let resp = Query::vault(&vault_entity_id)
        .await
        .expect("cannot get the query response");

    // Should include the deposits made in same vault entity
    for index in 0..deposits_config.len() {
        let deposit_id = format!("{:?}-{}", deposit_tx_hash, index);

        assert!(resp.deposits.contains(&deposit_id), "missing deposit id");
    }

    Ok(())
}

#[tokio::main]
#[test]
async fn vault_entity_clear() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);
    println!("alice.address(): {:?}", alice.address());
    println!("bob.address(): {:?}", bob.address());
    println!("bounty_bot.address(): {:?}", bounty_bot.address());

    let orderbook = get_orderbook().await.expect("cannot get OB");

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = touch_deployer(None).await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;
    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;
    // Generate vault ids for each account (Input and Output)
    let alice_input_vault = generate_random_u256();
    let alice_output_vault = generate_random_u256();
    let bob_input_vault = generate_random_u256();
    let bob_output_vault = generate_random_u256();
    let bounty_bot_vault_a = generate_random_u256();
    let bounty_bot_vault_b = generate_random_u256();

    // Order Alice Configuration
    let order_alice = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(alice_input_vault),
        &token_b,
        Some(alice_output_vault),
    )
    .await;

    // Order Bob Configuration
    let order_bob = generate_order_config(
        &expression_deployer,
        &token_b,
        Some(bob_input_vault),
        &token_a,
        Some(bob_output_vault),
    )
    .await;
    println!("Pre call add_order_alice");

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice
        .send()
        .await
        .expect("cannot send add order alice");
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await;
    println!(
        "alice sender: {:?} --- owner: {:?}",
        add_order_alice_data.sender, add_order_alice_data.order.owner
    );

    println!("Pre call add_order_bob");

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob
        .send()
        .await
        .expect("cannot send add order bob");
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await;
    println!(
        "bob sender: {:?} --- owner: {:?}",
        add_order_bob_data.sender, add_order_bob_data.order.owner
    );

    // Make deposit of corresponded output token
    let decimal_a = token_a.decimals().call().await?;
    let amount_alice = get_amount_tokens(8, decimal_a);

    let decimal_b = token_b.decimals().call().await?;
    let amount_bob = get_amount_tokens(6, decimal_b);

    // Alice has token_b as output
    mint_tokens(&amount_alice, &alice.address(), &token_b).await?;

    // Approve Alice token_b using to OB
    approve_tokens(
        // &amount_alice,
        &amount_alice,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await?;

    println!("Pre call deposit_func alice");
    // Deposit using Alice
    let deposit_func = orderbook.connect(&alice).await.deposit(
        token_b.address(),
        alice_output_vault,
        amount_alice,
    );
    let _ = deposit_func.send().await?.await?;

    // Bob has token_a as output
    mint_tokens(&amount_bob, &bob.address(), &token_a).await?;

    // Approve Bob token_a using to OB
    approve_tokens(
        &amount_bob,
        &orderbook.address(),
        &token_a.connect(&bob).await,
    )
    .await?;

    println!("Pre call deposit_func_2 bob");
    // Deposit using Bob
    let deposit_func_2 =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), bob_output_vault, amount_bob);
    let _ = deposit_func_2.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;
    let clear_config = generate_clear_config(&bounty_bot_vault_a, &bounty_bot_vault_b);

    println!("Pre call clear");

    let a_signed_context: Vec<generated::SignedContextV1> = Vec::new();
    let b_signed_context: Vec<generated::SignedContextV1> = Vec::new();

    let clear_func = orderbook.connect(&bounty_bot).await.clear(
        order_alice.to_owned(),
        order_bob.to_owned(),
        clear_config,
        a_signed_context,
        b_signed_context,
    );

    println!("Pre send clear");

    let tx = clear_func.send().await?;
    println!("Post send clear");

    let clear_data = get_clear_event(orderbook, &tx).await;
    println!("clear_data: {:?}\n", clear_data);

    let after_clear_data = get_after_clear_event(orderbook, &tx).await;
    println!("after_clear_data: {:?}\n", after_clear_data);

    // // Get a random vaultId
    // let vault_id = generate_random_u256();

    // // The expected vault entity SG ID
    // let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // ////////

    // // Build OrderConfig with the vaultId
    // let order_config = generate_order_config(
    //     &expression_deployer,
    //     &token_a,
    //     Some(vault_id),
    //     &token_b,
    //     Some(vault_id),
    // )
    // .await;

    // // Add the order
    // let add_order_func = orderbook.add_order(order_config.clone());
    // let _ = add_order_func
    //     .send()
    //     .await
    //     .expect("order not sent")
    //     .await
    //     .expect("cannot wait receipt");

    // // Wait for Subgraph sync
    // wait().await.expect("cannot get SG sync status");

    // // First query when adding order
    // let resp = Query::vault(&vault_entity_id)
    //     .await
    //     .expect("cannot get the query response");

    // // The whole entity should be created normally when adding the order
    // assert_eq!(resp.id, vault_entity_id);

    // // Now, make the deposits with a given amount
    // let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // // Fill to Alice with tokens (A and B)
    // mint_tokens(&amount, &alice.address(), &token_a)
    //     .await
    //     .expect("cannot mint tokens");

    // mint_tokens(&amount, &alice.address(), &token_b)
    //     .await
    //     .expect("cannot mint tokens");

    // // Connect token to Alice and approve Orderbook to move tokens
    // approve_tokens(
    //     &amount,
    //     &orderbook.address(),
    //     &token_a.connect(&alice).await,
    // )
    // .await
    // .expect("cannot approve tokens");

    // approve_tokens(
    //     &amount,
    //     &orderbook.address(),
    //     &token_b.connect(&alice).await,
    // )
    // .await
    // .expect("cannot approve tokens");

    // // Fill struct with same vaultId in the deposit configurations
    // let deposits_config = vec![
    //     // Config A
    //     TestDepositConfig {
    //         token: token_a.address(),
    //         vault_id: vault_id,
    //         amount,
    //     },
    //     // Config B
    //     TestDepositConfig {
    //         token: token_b.address(),
    //         vault_id: vault_id,
    //         amount,
    //     },
    // ];
    // // The multi deposit data bytes
    // let multi_deposit = generate_multi_deposit(&deposits_config);

    // // Send the deposits with multicall
    // let multicall_func = orderbook.multicall(multi_deposit);
    // let tx_multicall = multicall_func.send().await.expect("multicall not sent");
    // let tx_receipt = tx_multicall.await.expect("failed to wait receipt").unwrap();

    // let deposit_tx_hash = &tx_receipt.transaction_hash;

    // // Wait for Subgraph sync
    // wait().await.expect("cannot get SG sync status");

    // // Second query, using same vault entity ID.
    // let resp = Query::vault(&vault_entity_id)
    //     .await
    //     .expect("cannot get the query response");

    // // Should include the deposits made in same vault entity
    // for index in 0..deposits_config.len() {
    //     let deposit_id = format!("{:?}-{}", deposit_tx_hash, index);

    //     assert!(resp.deposits.contains(&deposit_id), "missing deposit id");
    // }

    Ok(())
}

// #[test]
fn util_cbor_meta_test() -> anyhow::Result<()> {
    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta: Vec<u8> = read_orderbook_meta();

    let output: Vec<RainMapDoc> = decode_rain_meta(ob_meta.clone().into())?;

    let encoded_again = encode_rain_docs(output);

    assert_eq!(ob_meta, encoded_again);

    Ok(())
}
