mod generated;
mod subgraph;
mod utils;

use std::ops::Div;

use ethers::{
    signers::Signer,
    types::{Address, Bytes, U256},
    utils::keccak256,
};
use subgraph::{wait, Query};
use utils::{
    bytes_to_h256,
    cbor::{decode_rain_meta, encode_rain_docs, RainMapDoc},
    deploy::{deploy_erc20_mock, get_expression_deployer, get_orderbook, read_orderbook_meta},
    events::{
        _get_new_expression_event, get_add_order_event, get_after_clear_events, get_clear_events,
        get_deposit_events, get_take_order_events, get_withdraw_events,
    },
    generate_random_u256, get_wallet, h256_to_bytes,
    json_structs::{NewExpressionJson, OrderJson},
    numbers::{display_number, get_amount_tokens},
    transactions::{
        approve_tokens, generate_clear_config, generate_multi_add_order, generate_multi_clear,
        generate_multi_deposit, generate_multi_withdraw, generate_order_config, get_block_data,
        get_decimals, mint_tokens, TestDepositConfig, TestWithdrawConfig,
    },
};

use generated::{ClearCall, SignedContextV1, TakeOrderConfig, TakeOrdersConfigV2};

#[tokio::main]
// #[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Wait for Subgraph sync
    wait().await?;

    // Query the OrderBook entity
    let response = Query::orderbook(&orderbook.address()).await?;

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
    let _ = get_orderbook().await?;

    // Wait for Subgraph sync
    wait().await?;

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta = read_orderbook_meta();
    let ob_meta_bytes = Bytes::from(ob_meta.clone());
    let ob_meta_hashed = Bytes::from(keccak256(ob_meta.clone()));
    let ob_meta_decoded = decode_rain_meta(ob_meta.clone().into())?;

    // Query the RainMetaV1 entity
    let response = Query::rain_meta_v1(&ob_meta_hashed.clone()).await?;

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
    let _ = get_orderbook().await?;

    // Wait for Subgraph sync
    wait().await?;

    // Read meta from root repository (output from nix command) and convert to Bytes
    let ob_meta = read_orderbook_meta();
    let ob_meta_hashed = Bytes::from(keccak256(ob_meta.clone()));
    let ob_meta_decoded = decode_rain_meta(ob_meta.clone().into())?;

    for content in ob_meta_decoded {
        // Query the ContentMetaV1 entity
        let response = Query::content_meta_v1(&content.hash().as_fixed_bytes().into()).await?;

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
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet
    let wallet_1 = get_wallet(1);
    let orderbook = orderbook.connect(&wallet_1).await;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());

    let tx_add_order = add_order_func.send().await?;

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await?;
    let new_expression_data =
        _get_new_expression_event(expression_deployer.clone(), &tx_add_order).await?;

    // Wait for Subgraph sync
    wait().await?;

    let order_hash = Bytes::from(add_order_data.order_hash);

    let response = Query::order(&order_hash).await?;

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
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet
    let wallet_1 = get_wallet(1);
    let orderbook = orderbook.connect(&wallet_1).await;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await?;

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await?;

    let order_hash = Bytes::from(add_order_data.order_hash);

    // Data from the event in tx
    let order_data = add_order_data.order;

    // Remove the order
    let remove_order_fnc = orderbook.remove_order(order_data);
    let _ = remove_order_fnc.send().await?;

    // Current order status
    let is_order_exist: bool = orderbook
        .order_exists(bytes_to_h256(&order_hash).into())
        .call()
        .await?;

    // Wait for Subgraph sync
    wait().await?;

    let response = Query::order(&order_hash).await?;

    assert_eq!(response.order_active, is_order_exist, "wrong order status");

    Ok(())
}

#[tokio::main]
// #[test]
async fn order_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

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

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), bob_output_vault, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;
    let clear_config = generate_clear_config(&bounty_bot_vault_a, &bounty_bot_vault_b);

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_func = orderbook.connect(&bounty_bot).await.clear(
        order_alice.to_owned(),
        order_bob.to_owned(),
        clear_config,
        a_signed_context,
        b_signed_context,
    );

    // Wait for the transaction
    let tx_clear = clear_func.send().await?;
    let clear_tx_hash = tx_clear.tx_hash();

    // Order hashes
    let alice_order_hash: Bytes = add_order_alice_data.order_hash.into();
    let bob_order_hash: Bytes = add_order_bob_data.order_hash.into();

    // Clear ID (using 0 since was only one clear)
    let clear_entity_id = format!("{:?}-{}", clear_tx_hash, 0);

    // Wait for Subgraph sync
    wait().await?;

    // Querying for both orders that should include the clearr
    let response_a = Query::order(&alice_order_hash).await?;
    let response_b = Query::order(&bob_order_hash).await?;

    assert!(
        response_a.orders_clears.contains(&clear_entity_id),
        "missing clear entity"
    );
    assert!(
        response_b.orders_clears.contains(&clear_entity_id),
        "missing clear entity"
    );

    Ok(())
}

#[tokio::main]
// #[test]
async fn io_entity_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Build OrderConfig
    let order_config =
        generate_order_config(&expression_deployer, &token_a, None, &token_b, None).await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await?;

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await?;

    // Order hash
    let order_hash = Bytes::from(add_order_data.order_hash);
    let order_owner: Address = add_order_data.order.owner;

    // Wait for Subgraph sync
    wait().await?;

    // Inputs
    for (index, input) in order_config.valid_inputs.iter().enumerate() {
        let token: Address = input.token;
        let vault_id: U256 = input.vault_id;
        let input_id = format!("{}-{:?}-{}", order_hash, token, vault_id);

        let vault_entity_id = format!("{}-{:?}", vault_id, order_owner);
        let token_vault_entity_id = format!("{}-{:?}-{:?}", vault_id, order_owner, token);

        let response = Query::i_o(&input_id).await?;

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

        let response = Query::i_o(&output_id).await?;

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
    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (C)
    let token_c = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (D)
    let token_d = deploy_erc20_mock(None).await?;

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
    let tx_multicall = multicall_func.send().await?;
    let _ = tx_multicall.await?;

    // Wait for Subgraph sync
    wait().await?;

    let vault_entity_id = format!("{}-{:?}", vault_id, wallet_owner.address());

    let response = Query::vault(&vault_entity_id).await?;

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
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens (A and B)
    mint_tokens(&amount, &alice.address(), &token_a).await?;

    mint_tokens(&amount, &alice.address(), &token_b).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await?;

    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await?;

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
    let tx_multicall = multicall_func.send().await?;
    let tx_receipt = tx_multicall.await?.unwrap();

    let deposit_tx_hash = &tx_receipt.transaction_hash;

    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Generate the expetect Token Vault IDs
    let token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());
    let token_vault_b = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_b.address());

    // Wait for Subgraph sync
    wait().await?;

    let resp = Query::vault(&vault_entity_id).await?;

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
async fn vault_entity_withdraw_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Vault Entity ID
    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Amount to deposit
    let amount_to_deposit = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens
    mint_tokens(&amount_to_deposit, &alice.address(), &token_a).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount_to_deposit,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await?;

    // Send the deposits with multicall
    let deposit_func = orderbook.deposit(token_a.address(), vault_id, amount_to_deposit);
    let _ = deposit_func.send().await?.await?;

    // Make two withdraw with the half of what was deposited
    let amount_to_withdaw = amount_to_deposit.div(2);

    // Fill struct with same vaultId and tokens in the Withdaws configurations
    let withdraws_config = vec![
        // Config A
        TestWithdrawConfig {
            token: token_a.address(),
            vault_id: vault_id,
            target_amount: amount_to_withdaw,
        },
        // Config B
        TestWithdrawConfig {
            token: token_a.address(),
            vault_id: vault_id,
            target_amount: amount_to_withdaw,
        },
    ];

    // Encode the withdaws
    let multi_withdaws = generate_multi_withdraw(&withdraws_config);

    // Send the deposits with multicall
    let multicall_func = orderbook.multicall(multi_withdaws);
    let tx_multicall_withdraws = multicall_func.send().await?;
    let withdraw_tx_hash = tx_multicall_withdraws.tx_hash();

    // Generate the expetect Token Vault IDs
    let token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());

    // Wait for Subgraph sync
    wait().await?;

    let resp = Query::vault(&vault_entity_id).await?;

    // The whole entity should be updated normally
    assert_eq!(resp.vault_id, vault_id);
    assert_eq!(resp.owner, alice.address());
    assert!(
        resp.token_vaults.contains(&token_vault_a),
        "Missing tokenVault id"
    );

    // Should include the withdraws made
    for index in 0..withdraws_config.len() {
        let withdraws_id = format!("{:?}-{}", withdraw_tx_hash, index);

        assert!(
            resp.withdraws.contains(&withdraws_id),
            "missing withdraw id"
        );
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_entity_add_order_and_deposit_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // The expected vault entity SG ID
    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

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
    let _ = add_order_func.send().await?.await?;

    // Wait for Subgraph sync
    wait().await?;

    // First query when adding order
    let resp = Query::vault(&vault_entity_id).await?;

    // The whole entity should be created normally when adding the order
    assert_eq!(resp.id, vault_entity_id);

    // Now, make the deposits with a given amount
    let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens (A and B)
    mint_tokens(&amount, &alice.address(), &token_a).await?;

    mint_tokens(&amount, &alice.address(), &token_b).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await?;

    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await?;

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
    let tx_multicall = multicall_func.send().await?;
    let tx_receipt = tx_multicall.await?.unwrap();

    let deposit_tx_hash = &tx_receipt.transaction_hash;

    // Wait for Subgraph sync
    wait().await?;

    // Second query, using same vault entity ID.
    let resp = Query::vault(&vault_entity_id).await?;

    // Should include the deposits made in same vault entity
    for index in 0..deposits_config.len() {
        let deposit_id = format!("{:?}-{}", deposit_tx_hash, index);

        assert!(resp.deposits.contains(&deposit_id), "missing deposit id");
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

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

    // The vaultIds entities for bounty account
    let vault_a_entity_id = format!("{}-{:?}", bounty_bot_vault_a, bounty_bot.address());
    let vault_b_entity_id = format!("{}-{:?}", bounty_bot_vault_b, bounty_bot.address());

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

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), bob_output_vault, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;
    let clear_config = generate_clear_config(&bounty_bot_vault_a, &bounty_bot_vault_b);

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_func = orderbook.connect(&bounty_bot).await.clear(
        order_alice.to_owned(),
        order_bob.to_owned(),
        clear_config,
        a_signed_context,
        b_signed_context,
    );

    // Wait for the transaction
    let _ = clear_func.send().await?.await?;

    let token_vault_bounty_a = format!(
        "{}-{:?}-{:?}",
        bounty_bot_vault_a,
        bounty_bot.address(),
        token_b.address(), // Using the output of alice order
    );

    let token_vault_bounty_b = format!(
        "{}-{:?}-{:?}",
        bounty_bot_vault_b,
        bounty_bot.address(),
        token_a.address(), // Using the output of bob order
    );

    // Wait for Subgraph sync
    wait().await?;

    // Querying for both vaults
    let resp_a = Query::vault(&vault_a_entity_id).await?;
    let resp_b = Query::vault(&vault_b_entity_id).await?;

    // BountyAlice Vault
    assert_eq!(resp_a.owner, bounty_bot.address());
    assert_eq!(resp_a.vault_id, bounty_bot_vault_a);
    assert!(resp_a.token_vaults.contains(&token_vault_bounty_a));

    // BountyBob Vault
    assert_eq!(resp_b.owner, bounty_bot.address());
    assert_eq!(resp_b.vault_id, bounty_bot_vault_b);
    assert!(resp_b.token_vaults.contains(&token_vault_bounty_b));

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_deposit_multiple_deposits_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    let amount = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens (A and B)
    mint_tokens(&amount, &alice.address(), &token_a).await?;

    mint_tokens(&amount, &alice.address(), &token_b).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await?;

    approve_tokens(
        &amount,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await?;

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
    let tx_multicall = multicall_func.send().await?;
    let tx_receipt = tx_multicall.await?.unwrap();

    let deposits_tx_hash = &tx_receipt.transaction_hash;

    let block_data = get_block_data(&deposits_tx_hash).await?;

    let deposit_events = get_deposit_events(&orderbook, &deposits_tx_hash).await?;

    // Wait for Subgraph sync
    wait().await?;

    for (index, deposit) in deposit_events.iter().enumerate() {
        let deposit_id = format!("{:?}-{}", deposits_tx_hash, index);

        let vault_entity_id = format!("{}-{:?}", deposit.vault_id, alice.address());
        let amount_display = display_number(deposit.amount, get_decimals(deposit.token).await?);

        let token_vault_entity = format!(
            "{}-{:?}-{:?}",
            deposit.vault_id,
            alice.address(),
            deposit.token,
        );

        let resp = Query::vault_deposit(&deposit_id).await?;

        assert_eq!(resp.sender, alice.address());
        assert_eq!(resp.token, deposit.token);
        assert_eq!(resp.vault_id, deposit.vault_id);
        assert_eq!(resp.vault, vault_entity_id);
        assert_eq!(resp.amount, deposit.amount);
        assert_eq!(resp.amount_display, amount_display);
        assert_eq!(resp.token_vault, token_vault_entity);
        assert_eq!(resp.transaction, *deposits_tx_hash);
        assert_eq!(resp.emitter, alice.address());
        assert_eq!(resp.timestamp, block_data.timestamp);
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn vault_withdraw_multiple_withdraws_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Amount to deposit
    let amount_to_deposit = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());

    // Fill to Alice with tokens
    mint_tokens(&amount_to_deposit, &alice.address(), &token_a).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount_to_deposit,
        &orderbook.address(),
        &token_a.connect(&alice).await,
    )
    .await?;

    // Send the deposits with multicall
    let deposit_func = orderbook.deposit(token_a.address(), vault_id, amount_to_deposit);
    let _ = deposit_func.send().await?.await?;

    // Make two withdraw with the half of what was deposited
    let amount_to_withdaw = amount_to_deposit.div(2);

    // Fill struct with same vaultId and tokens in the Withdaws configurations
    let withdraws_config = vec![
        // Config A
        TestWithdrawConfig {
            token: token_a.address(),
            vault_id: vault_id,
            target_amount: amount_to_withdaw,
        },
        // Config B
        TestWithdrawConfig {
            token: token_a.address(),
            vault_id: vault_id,
            target_amount: amount_to_withdaw,
        },
    ];

    // Encode the withdaws
    let multi_withdaws = generate_multi_withdraw(&withdraws_config);

    // Send the deposits with multicall
    let multicall_func = orderbook.multicall(multi_withdaws);
    let tx_multicall_withdraws = multicall_func.send().await?;
    let withdraw_tx_hash = tx_multicall_withdraws.tx_hash();

    let block_data = get_block_data(&withdraw_tx_hash).await?;

    let withdraw_events = get_withdraw_events(&orderbook, &withdraw_tx_hash).await?;

    // Wait for Subgraph sync
    wait().await?;

    for (index, withdraw) in withdraw_events.iter().enumerate() {
        let withdraw_id = format!("{:?}-{}", withdraw_tx_hash, index);

        let vault_entity_id = format!("{}-{:?}", withdraw.vault_id, alice.address());
        let decimals = get_decimals(withdraw.token).await?;
        let amount_display = display_number(withdraw.amount, decimals);

        let requested_amount_display = display_number(withdraw.target_amount, decimals);

        let token_vault_entity = format!(
            "{}-{:?}-{:?}",
            withdraw.vault_id,
            alice.address(),
            withdraw.token,
        );

        let resp = Query::vault_withdraw(&withdraw_id).await?;

        assert_eq!(resp.sender, alice.address());
        assert_eq!(resp.token, withdraw.token);
        assert_eq!(resp.vault_id, withdraw.vault_id);
        assert_eq!(resp.vault, vault_entity_id);
        assert_eq!(resp.requested_amount, withdraw.target_amount);
        assert_eq!(resp.requested_amount_display, requested_amount_display);
        assert_eq!(resp.amount, withdraw.amount);
        assert_eq!(resp.amount_display, amount_display);
        assert_eq!(resp.token_vault, token_vault_entity);
        assert_eq!(resp.transaction, withdraw_tx_hash);
        assert_eq!(resp.emitter, alice.address());
        assert_eq!(resp.timestamp, block_data.timestamp);
    }

    Ok(())
}

#[tokio::main]
// #[test]
async fn erc20_entity_add_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

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
    let _ = add_order_func.send().await?.await?;

    // Wait for Subgraph sync
    wait().await?;

    // Both ERC20 entities should be created
    let resp_a = Query::erc20(&token_a.address()).await?;
    let resp_b = Query::erc20(&token_b.address()).await?;

    let total_supply_display_a = display_number(
        token_a.total_supply().call().await?,
        token_a.decimals().call().await?,
    );

    let total_supply_display_b = display_number(
        token_b.total_supply().call().await?,
        token_b.decimals().call().await?,
    );

    // Checking the token A
    assert_eq!(resp_a.name, token_a.name().call().await?);
    assert_eq!(resp_a.symbol, token_a.symbol().call().await?);
    assert_eq!(resp_a.total_supply, token_a.total_supply().call().await?);
    assert_eq!(resp_a.decimals, token_a.decimals().call().await?);
    assert_eq!(resp_a.total_supply_display, total_supply_display_a);

    // Checking the token B
    assert_eq!(resp_b.name, token_b.name().call().await?);
    assert_eq!(resp_b.symbol, token_b.symbol().call().await?);
    assert_eq!(resp_b.total_supply, token_b.total_supply().call().await?);
    assert_eq!(resp_b.decimals, token_b.decimals().call().await?);
    assert_eq!(resp_b.total_supply_display, total_supply_display_b);

    Ok(())
}

#[tokio::main]
// #[test]
async fn erc20_entity_deposit_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ERC20 token contract
    let token = deploy_erc20_mock(None).await?;

    // Now, make the deposits with a given amount
    let amount = get_amount_tokens(1000, token.decimals().call().await?);

    // Fill to Alice with tokens
    mint_tokens(&amount, &alice.address(), &token).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(&amount, &orderbook.address(), &token.connect(&alice).await).await?;

    // Send the deposits
    let deposit_func = orderbook.deposit(token.address(), vault_id, amount);
    let _ = deposit_func.send().await?.await?;

    // Wait for Subgraph sync
    wait().await?;

    // Second query, using same vault entity ID.
    let resp = Query::erc20(&token.address()).await?;

    let total_supply_display = display_number(
        token.total_supply().call().await?,
        token.decimals().call().await?,
    );

    // Checking the token
    assert_eq!(resp.name, token.name().call().await?);
    assert_eq!(resp.symbol, token.symbol().call().await?);
    assert_eq!(resp.total_supply, token.total_supply().call().await?);
    assert_eq!(resp.decimals, token.decimals().call().await?);
    assert_eq!(resp.total_supply_display, total_supply_display);

    Ok(())
}

#[tokio::main]
// #[test]
async fn order_clear_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

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

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), bob_output_vault, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;

    let alice_hash = add_order_alice_data.order_hash;
    let bob_hash = add_order_bob_data.order_hash;

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_config_1 = generate_clear_config(&bounty_bot_vault_a, &bounty_bot_vault_b);
    let clear_1 = ClearCall {
        alice: order_alice.to_owned(),
        bob: order_bob.to_owned(),
        clear_config: clear_config_1,
        alice_signed_context: a_signed_context.clone(),
        bob_signed_context: b_signed_context.clone(),
    };

    let clear_config_2 = generate_clear_config(&bounty_bot_vault_b, &bounty_bot_vault_a);
    let clear_2 = ClearCall {
        alice: order_bob.to_owned(),
        bob: order_alice.to_owned(),
        clear_config: clear_config_2,
        alice_signed_context: b_signed_context,
        bob_signed_context: a_signed_context,
    };

    let clear_configs = vec![clear_1, clear_2];

    let multi_clear_bytes = generate_multi_clear(&clear_configs);

    let multicall_func = orderbook
        .connect(&bounty_bot)
        .await
        .multicall(multi_clear_bytes);

    let tx_multicall = multicall_func.send().await?;

    // Tx hash that hold all the logs
    let clears_tx_hash = tx_multicall.tx_hash();

    let clear_events = get_clear_events(&orderbook, &clears_tx_hash).await?;
    let after_clear_events = get_after_clear_events(&orderbook, &clears_tx_hash).await?;

    let block_data = get_block_data(&clears_tx_hash).await?;

    // It should emit the same amount of events both parts
    assert_eq!(clear_events.len(), after_clear_events.len());

    // Wait for Subgraph sync
    wait().await?;

    for (index, clear) in clear_events.iter().enumerate() {
        let order_clear_id = format!("{:?}-{}", clears_tx_hash, index);
        let bounty_id = order_clear_id.clone();
        let state_change_id = order_clear_id.clone();

        let resp = Query::order_clear(&order_clear_id).await?;

        assert_eq!(resp.sender, bounty_bot.address());
        assert_eq!(resp.clearer, bounty_bot.address());
        assert_eq!(resp.emitter, bounty_bot.address());

        // The "Alice" and "Bob" orders names are from the name inside the contract. Not related to names in tests
        // If the "ALICE" in the order come from alice wallet, then it's first order in event
        // otherwise, it's the second
        if clear.alice.owner == alice.address() {
            assert_eq!(resp.order_a, h256_to_bytes(&alice_hash.into()));
        } else {
            assert_eq!(resp.order_b, h256_to_bytes(&alice_hash.into()));
        }

        // If the "BOB" in the order come from bob wallet, then it's second order in event
        // otherwise, it's the first
        if clear.bob.owner == bob.address() {
            assert_eq!(resp.order_b, h256_to_bytes(&bob_hash.into()));
        } else {
            assert_eq!(resp.order_a, h256_to_bytes(&bob_hash.into()));
        }

        assert_eq!(resp.bounty, bounty_id);
        assert_eq!(resp.state_change, state_change_id);

        assert_eq!(resp.transaction, clears_tx_hash);
        assert_eq!(resp.timestamp, block_data.timestamp);

        assert!(resp.owners.contains(&clear.alice.owner));
        assert!(resp.owners.contains(&clear.bob.owner));

        assert_eq!(
            resp.a_input_io_index,
            clear.clear_config.alice_input_io_index
        );
        assert_eq!(
            resp.a_output_io_index,
            clear.clear_config.alice_output_io_index
        );

        assert_eq!(resp.b_input_io_index, clear.clear_config.bob_input_io_index);
        assert_eq!(
            resp.b_output_io_index,
            clear.clear_config.bob_output_io_index
        );
    }
    Ok(())
}

#[tokio::main]
// #[test]
async fn token_vault_entity_add_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(5);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

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
    let tx = add_order_func.send().await?;

    let add_order_data = get_add_order_event(&orderbook, &tx.tx_hash()).await?;
    let order_hash: Bytes = add_order_data.order_hash.into();

    // Wait for Subgraph sync
    wait().await?;

    // Generate the both Token Vault IDs
    let token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());
    let token_vault_b = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_b.address());

    // Vault entity ID
    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Vault Balances for both
    let vault_balance_a: U256 = orderbook
        .vault_balance(alice.address(), token_a.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_a =
        display_number(vault_balance_a, get_decimals(token_a.address()).await?);

    let vault_balance_b: U256 = orderbook
        .vault_balance(alice.address(), token_b.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_b =
        display_number(vault_balance_b, get_decimals(token_b.address()).await?);

    // Both ERC20 entities should be created
    let resp_a = Query::token_vault(&token_vault_a).await?;
    let resp_b = Query::token_vault(&token_vault_b).await?;

    // Checking token vault A
    assert_eq!(resp_a.owner, alice.address());
    assert_eq!(resp_a.vault, vault_entity_id);
    assert_eq!(resp_a.vault_id, vault_id);
    assert_eq!(resp_a.token, token_a.address());
    assert_eq!(resp_a.balance, vault_balance_a);
    assert_eq!(resp_a.balance_display, vaul_balance_display_a);
    assert!(resp_a.orders.contains(&order_hash), "missing order ID");

    // Checking token vault B
    assert_eq!(resp_b.owner, alice.address());
    assert_eq!(resp_b.vault, vault_entity_id);
    assert_eq!(resp_b.vault_id, vault_id);
    assert_eq!(resp_b.token, token_b.address());
    assert_eq!(resp_b.balance, vault_balance_b);
    assert_eq!(resp_b.balance_display, vaul_balance_display_b);
    assert!(resp_b.orders.contains(&order_hash), "missing order ID");

    Ok(())
}

#[tokio::main]
// #[test]
async fn token_vault_entity_deposit_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ERC20 token contract
    let token = deploy_erc20_mock(None).await?;

    // Now, make the deposits with a given amount
    let amount = get_amount_tokens(1000, token.decimals().call().await?);

    // Fill to Alice with tokens
    mint_tokens(&amount, &alice.address(), &token).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(&amount, &orderbook.address(), &token.connect(&alice).await).await?;

    // Send the deposits
    let deposit_func = orderbook.deposit(token.address(), vault_id, amount);
    let _ = deposit_func.send().await?.await?;

    // Wait for Subgraph sync
    wait().await?;

    let token_vault = format!("{}-{:?}-{:?}", vault_id, alice.address(), token.address());

    // Vault entity ID
    let vault_entity_id = format!("{}-{:?}", vault_id, alice.address());

    // Vault Balance
    let vault_balance: U256 = orderbook
        .vault_balance(alice.address(), token.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display = display_number(vault_balance, get_decimals(token.address()).await?);

    // Second query, using same vault entity ID.
    let resp = Query::token_vault(&token_vault).await?;

    // Checking token vault
    assert_eq!(resp.owner, alice.address());
    assert_eq!(resp.vault, vault_entity_id);
    assert_eq!(resp.vault_id, vault_id);
    assert_eq!(resp.token, token.address());
    assert_eq!(resp.balance, vault_balance);
    assert_eq!(resp.balance_display, vaul_balance_display);

    Ok(())
}

#[tokio::main]
// #[test]
async fn token_vault_entity_withdraw_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    // Connect the orderbook to another wallet (arbitrary) to send the orders
    let alice = get_wallet(2);
    let orderbook = orderbook.connect(&alice).await;

    // Get a random vaultId
    let vault_id = generate_random_u256();

    // Deploy ERC20 token contract (A)
    let token = deploy_erc20_mock(None).await?;

    // Amount to deposit
    let amount_to_deposit = get_amount_tokens(1000, token.decimals().call().await.unwrap());

    // Fill to Alice with tokens
    mint_tokens(&amount_to_deposit, &alice.address(), &token).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount_to_deposit,
        &orderbook.address(),
        &token.connect(&alice).await,
    )
    .await?;

    // Send the deposits with multicall
    let deposit_func = orderbook.deposit(token.address(), vault_id, amount_to_deposit);
    let _ = deposit_func.send().await?.await?;

    // Fill struct
    let withdraws_config = vec![
        // Config A
        TestWithdrawConfig {
            token: token.address(),
            vault_id: vault_id,
            target_amount: amount_to_deposit.div(2),
        },
    ];

    // Encode the withdaws
    let multi_withdaws = generate_multi_withdraw(&withdraws_config);

    // Send the withdaw with multicall
    let multicall_func = orderbook.multicall(multi_withdaws);
    let _ = multicall_func.send().await?.await?;

    // Generate the expetect Token Vault IDs
    let token_vault = format!("{}-{:?}-{:?}", vault_id, alice.address(), token.address());

    // Vault Balance
    let vault_balance: U256 = orderbook
        .vault_balance(alice.address(), token.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display = display_number(vault_balance, get_decimals(token.address()).await?);

    // Wait for Subgraph sync
    wait().await?;

    let resp = Query::token_vault(&token_vault).await?;

    // Checking token vault
    assert_eq!(resp.token, token.address());
    assert_eq!(resp.balance, vault_balance);
    assert_eq!(resp.balance_display, vaul_balance_display);

    Ok(())
}

#[tokio::main]
// #[test]
async fn token_vault_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let carl = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;
    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Generate vault ids for each account (same for all accounts)
    let vault_id = generate_random_u256();

    // Order Alice Configuration
    let order_alice = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;

    // Order Bob Configuration
    let order_bob = generate_order_config(
        &expression_deployer,
        &token_b,
        Some(vault_id),
        &token_a,
        Some(vault_id),
    )
    .await;

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Alice
    let deposit_func =
        orderbook
            .connect(&alice)
            .await
            .deposit(token_b.address(), vault_id, amount_alice);
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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), vault_id, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;
    let clear_config = generate_clear_config(&vault_id, &vault_id);

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_func = orderbook.connect(&carl).await.clear(
        order_alice.to_owned(),
        order_bob.to_owned(),
        clear_config,
        a_signed_context,
        b_signed_context,
    );

    // Wait for the transaction
    let tx_clear = clear_func.send().await?;
    let clear_tx_hash = tx_clear.tx_hash();

    // Clear ID (using 0 since was only one clear)
    let clear_entity_id = format!("{:?}-{}", clear_tx_hash, 0);

    // Wait for Subgraph sync
    wait().await?;

    // Generate the both Token Vault IDs
    let alice_token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());
    let alice_token_vault_b = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_b.address());
    let bob_token_vault_a = format!("{}-{:?}-{:?}", vault_id, bob.address(), token_a.address());
    let bob_token_vault_b = format!("{}-{:?}-{:?}", vault_id, bob.address(), token_b.address());
    let carl_token_vault_a = format!("{}-{:?}-{:?}", vault_id, carl.address(), token_a.address());
    let carl_token_vault_b = format!("{}-{:?}-{:?}", vault_id, carl.address(), token_b.address());

    // All token vault entities present on the Clear should be created
    // - ALICE
    let vault_balance_a: U256 = orderbook
        .vault_balance(alice.address(), token_a.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_a =
        display_number(vault_balance_a, get_decimals(token_a.address()).await?);

    let vault_balance_b: U256 = orderbook
        .vault_balance(alice.address(), token_b.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_b =
        display_number(vault_balance_b, get_decimals(token_b.address()).await?);

    let resp_a = Query::token_vault(&alice_token_vault_a).await?;
    let resp_b = Query::token_vault(&alice_token_vault_b).await?;

    assert_eq!(resp_a.owner, alice.address());
    assert_eq!(resp_a.balance, vault_balance_a);
    assert_eq!(resp_a.balance_display, vaul_balance_display_a);
    assert!(
        resp_a.orders_clears.contains(&clear_entity_id),
        "missing clear ID"
    );

    assert_eq!(resp_b.owner, alice.address());
    assert_eq!(resp_b.balance, vault_balance_b);
    assert_eq!(resp_b.balance_display, vaul_balance_display_b);
    assert!(
        resp_b.orders_clears.contains(&clear_entity_id),
        "missing clear ID"
    );

    // - BOB
    let vault_balance_a: U256 = orderbook
        .vault_balance(bob.address(), token_a.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_a =
        display_number(vault_balance_a, get_decimals(token_a.address()).await?);

    let vault_balance_b: U256 = orderbook
        .vault_balance(bob.address(), token_b.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_b =
        display_number(vault_balance_b, get_decimals(token_b.address()).await?);

    let resp_a = Query::token_vault(&bob_token_vault_a).await?;
    let resp_b = Query::token_vault(&bob_token_vault_b).await?;

    assert_eq!(resp_a.owner, bob.address());
    assert_eq!(resp_a.balance, vault_balance_a);
    assert_eq!(resp_a.balance_display, vaul_balance_display_a);
    assert!(
        resp_a.orders_clears.contains(&clear_entity_id),
        "missing clear ID"
    );

    assert_eq!(resp_b.owner, bob.address());
    assert_eq!(resp_b.balance, vault_balance_b);
    assert_eq!(resp_b.balance_display, vaul_balance_display_b);
    assert!(
        resp_b.orders_clears.contains(&clear_entity_id),
        "missing clear ID"
    );

    // - CARL
    let vault_balance_a: U256 = orderbook
        .vault_balance(carl.address(), token_a.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_a =
        display_number(vault_balance_a, get_decimals(token_a.address()).await?);

    let vault_balance_b: U256 = orderbook
        .vault_balance(carl.address(), token_b.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_b =
        display_number(vault_balance_b, get_decimals(token_b.address()).await?);

    let resp_a = Query::token_vault(&carl_token_vault_a).await?;
    let resp_b = Query::token_vault(&carl_token_vault_b).await?;

    assert_eq!(resp_a.owner, carl.address());
    assert_eq!(resp_a.balance, vault_balance_a);
    assert_eq!(resp_a.balance_display, vaul_balance_display_a);

    assert_eq!(resp_b.owner, carl.address());
    assert_eq!(resp_b.balance, vault_balance_b);
    assert_eq!(resp_b.balance_display, vaul_balance_display_b);

    Ok(())
}

#[tokio::main]
// #[test]
async fn token_vault_entity_take_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    let alice = get_wallet(1);
    let bob = get_wallet(2);

    // Connect the orderbook to another wallet
    let orderbook = orderbook.connect(&alice).await;

    // Vault id
    let vault_id = generate_random_u256();

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;
    // let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A) connected to Alice
    let token_a = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B) connected to Alice
    let token_b = deploy_erc20_mock(None).await?;

    // Build OrderConfig
    let order_config = generate_order_config(
        expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await?;

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await?;

    // Amount to deposit
    let amount_b = get_amount_tokens(1000, token_b.decimals().call().await.unwrap());

    // Fill to Alice with tokens
    mint_tokens(&amount_b, &alice.address(), &token_b).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount_b,
        &orderbook.address(),
        &token_b.connect(&alice).await,
    )
    .await?;

    // Alice deposit tokens
    let deposit_func = orderbook.deposit(token_b.address(), vault_id, amount_b);
    let _ = deposit_func.send().await?;

    // BOB TAKE THE ORDER

    // Take Order configs
    let minimum_input = U256::from(0);
    let maximum_input = get_amount_tokens(1000, token_b.decimals().call().await.unwrap());
    let maximum_io_ratio = U256::from(10000000000000000000u64); // 10e18

    let take_order_config = TakeOrderConfig {
        order: add_order_data.order,
        input_io_index: U256::zero(),
        output_io_index: U256::zero(),
        signed_context: Vec::new(),
    };

    let take_orders_config = TakeOrdersConfigV2 {
        minimum_input,
        maximum_input,
        maximum_io_ratio,
        orders: vec![take_order_config],
        data: Bytes::new(), // Empty data
    };

    // Fill bob with token A (token input of the order)
    // let amount_a = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());
    let amount_a = amount_b
        .saturating_mul(maximum_io_ratio)
        .checked_div(U256::from(1000000000000000000u64)) //  1e18
        .unwrap();

    mint_tokens(&amount_a, &bob.address(), &token_a).await?;

    // Connect token to Bob and approve Orderbook to move tokens
    approve_tokens(
        &amount_a,
        &orderbook.address(),
        &token_a.connect(&bob).await,
    )
    .await?;

    // Take the order
    let take_order_func = orderbook
        .connect(&bob)
        .await
        .take_orders(take_orders_config);
    let tx_take_order = take_order_func.send().await?;

    let tx_receipt = tx_take_order.await?.unwrap();

    let take_order_tx_hash = &tx_receipt.transaction_hash;

    let take_order_events = get_take_order_events(&orderbook, &take_order_tx_hash).await?;

    // Just one take order happned in this transaction
    assert!(take_order_events.len() == 1);

    let token_vault_a = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_a.address());
    let token_vault_b = format!("{}-{:?}-{:?}", vault_id, alice.address(), token_b.address());

    // Index 0 since only one take order was made in this tx
    let take_order_entity_a_id = format!("{:?}-{}-{}", take_order_tx_hash, 0, token_vault_a);
    let take_order_entity_b_id = format!("{:?}-{}-{}", take_order_tx_hash, 0, token_vault_b);

    // Vault Balances for both
    let vault_balance_a: U256 = orderbook
        .vault_balance(alice.address(), token_a.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_a =
        display_number(vault_balance_a, get_decimals(token_a.address()).await?);

    let vault_balance_b: U256 = orderbook
        .vault_balance(alice.address(), token_b.address(), vault_id)
        .call()
        .await?;
    let vaul_balance_display_b =
        display_number(vault_balance_b, get_decimals(token_b.address()).await?);

    // Both entities should be updated
    let resp_a = Query::token_vault(&token_vault_a).await?;
    let resp_b = Query::token_vault(&token_vault_b).await?;

    // Checking token vault A
    assert_eq!(resp_a.owner, alice.address());
    assert_eq!(resp_a.vault_id, vault_id);
    assert_eq!(resp_a.token, token_a.address());
    assert_eq!(resp_a.balance, vault_balance_a);
    assert_eq!(resp_a.balance_display, vaul_balance_display_a);
    assert!(
        resp_a.take_orders.contains(&take_order_entity_a_id),
        "missing take order ID"
    );

    // Checking token vault B
    assert_eq!(resp_b.owner, alice.address());
    assert_eq!(resp_b.vault_id, vault_id);
    assert_eq!(resp_b.token, token_b.address());
    assert_eq!(resp_b.balance, vault_balance_b);
    assert_eq!(resp_b.balance_display, vaul_balance_display_b);
    assert!(
        resp_b.take_orders.contains(&take_order_entity_b_id),
        "missing take order ID"
    );

    Ok(())
}

#[tokio::main]
// #[test]
async fn bounty_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;
    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Generate vault ids for each account (Input and Output)
    let vault_id = generate_random_u256();

    // Order Alice Configuration
    let order_alice = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;

    // Order Bob Configuration
    let order_bob = generate_order_config(
        &expression_deployer,
        &token_b,
        Some(vault_id),
        &token_a,
        Some(vault_id),
    )
    .await;

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Alice
    let deposit_func =
        orderbook
            .connect(&alice)
            .await
            .deposit(token_b.address(), vault_id, amount_alice);
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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), vault_id, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_config_1 = generate_clear_config(&vault_id, &vault_id);
    let clear_1 = ClearCall {
        alice: order_alice.to_owned(),
        bob: order_bob.to_owned(),
        clear_config: clear_config_1,
        alice_signed_context: a_signed_context.clone(),
        bob_signed_context: b_signed_context.clone(),
    };

    let clear_config_2 = generate_clear_config(&vault_id, &vault_id);
    let clear_2 = ClearCall {
        alice: order_bob.to_owned(),
        bob: order_alice.to_owned(),
        clear_config: clear_config_2,
        alice_signed_context: b_signed_context,
        bob_signed_context: a_signed_context,
    };

    let clear_configs = vec![clear_1, clear_2];

    let multi_clear_bytes = generate_multi_clear(&clear_configs);

    let multicall_func = orderbook
        .connect(&bounty_bot)
        .await
        .multicall(multi_clear_bytes);

    let tx_multicall = multicall_func.send().await?;

    // Tx hash that hold all the logs
    let clears_tx_hash = tx_multicall.tx_hash();

    let clear_events = get_clear_events(&orderbook, &clears_tx_hash).await?;
    let after_clear_events = get_after_clear_events(&orderbook, &clears_tx_hash).await?;

    let block_data = get_block_data(&clears_tx_hash).await?;

    // It should emit the same amount of events both parts
    assert_eq!(clear_events.len(), after_clear_events.len());

    // Bounty Vault Entity ID
    let bounty_vault = format!("{}-{:?}", vault_id, bounty_bot.address());

    // Wait for Subgraph sync
    wait().await?;

    for (index, clear) in clear_events.iter().enumerate() {
        let after_clear = after_clear_events.get(index).unwrap();

        let bounty_entity_id = format!("{:?}-{}", clears_tx_hash, index);
        let order_clear_id = bounty_entity_id.clone();

        let clear_state_change = &after_clear.clear_state_change;

        // In these tests, generally only one token is added in the Order, so we pick the "first" in the array
        let alice_token_output: &Address = &clear.alice.valid_outputs.first().unwrap().token;
        let bob_token_output: &Address = &clear.bob.valid_outputs.first().unwrap().token;

        // Bounty Amount from A (alice)
        let bounty_amount_a = clear_state_change
            .alice_output
            .saturating_sub(clear_state_change.bob_input);

        let bounty_amount_a_display =
            display_number(bounty_amount_a, get_decimals(*alice_token_output).await?);

        // Bounty Amount from B (bpb)
        let bounty_amount_b = clear_state_change
            .bob_output
            .saturating_sub(clear_state_change.alice_input);

        let bounty_amount_b_display =
            display_number(bounty_amount_b, get_decimals(*bob_token_output).await?);

        let resp = Query::bounty(&bounty_entity_id).await?;

        assert_eq!(resp.clearer, bounty_bot.address());
        assert_eq!(resp.order_clear, order_clear_id);

        assert_eq!(resp.bounty_vault_a, bounty_vault);
        assert_eq!(resp.bounty_vault_b, bounty_vault);

        assert_eq!(resp.bounty_token_a, *alice_token_output);
        assert_eq!(resp.bounty_token_b, *bob_token_output);

        assert_eq!(resp.bounty_amount_a, Some(bounty_amount_a));
        assert_eq!(resp.bounty_amount_a_display, Some(bounty_amount_a_display));

        assert_eq!(resp.bounty_amount_b, Some(bounty_amount_b));
        assert_eq!(resp.bounty_amount_b_display, Some(bounty_amount_b_display));

        assert_eq!(resp.transaction, clears_tx_hash);
        assert_eq!(resp.emitter, bounty_bot.address());
        assert_eq!(resp.timestamp, block_data.timestamp);
    }
    Ok(())
}

#[tokio::main]
// #[test]
async fn order_clear_state_change_entity_clear_test() -> anyhow::Result<()> {
    let alice = get_wallet(0);
    let bob = get_wallet(1);
    let bounty_bot = get_wallet(2);

    let orderbook = get_orderbook().await?;

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A)
    let token_a = deploy_erc20_mock(None).await?;
    // Deploy ERC20 token contract (B)
    let token_b = deploy_erc20_mock(None).await?;

    // Generate vault ids for each account (Input and Output)
    let vault_id = generate_random_u256();

    // Order Alice Configuration
    let order_alice = generate_order_config(
        &expression_deployer,
        &token_a,
        Some(vault_id),
        &token_b,
        Some(vault_id),
    )
    .await;

    // Order Bob Configuration
    let order_bob = generate_order_config(
        &expression_deployer,
        &token_b,
        Some(vault_id),
        &token_a,
        Some(vault_id),
    )
    .await;

    // Add order alice with Alice connected to the OB
    let add_order_alice = orderbook.connect(&alice).await.add_order(order_alice);
    let tx = add_order_alice.send().await?;
    let add_order_alice_data = get_add_order_event(orderbook, &tx).await?;

    // Add order bob with Bob connected to the OB
    let add_order_bob = orderbook.connect(&bob).await.add_order(order_bob);
    let tx = add_order_bob.send().await?;
    let add_order_bob_data = get_add_order_event(orderbook, &tx).await?;

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

    // Deposit using Alice
    let deposit_func =
        orderbook
            .connect(&alice)
            .await
            .deposit(token_b.address(), vault_id, amount_alice);
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

    // Deposit using Bob
    let deposit_func =
        orderbook
            .connect(&bob)
            .await
            .deposit(token_a.address(), vault_id, amount_bob);
    let _ = deposit_func.send().await?.await?;

    // BOUNTY BOT CLEARS THE ORDER
    // Clear configuration
    let order_alice = &add_order_alice_data.order;
    let order_bob = &add_order_bob_data.order;

    let a_signed_context: Vec<SignedContextV1> = Vec::new();
    let b_signed_context: Vec<SignedContextV1> = Vec::new();

    let clear_config_1 = generate_clear_config(&vault_id, &vault_id);
    let clear_1 = ClearCall {
        alice: order_alice.to_owned(),
        bob: order_bob.to_owned(),
        clear_config: clear_config_1,
        alice_signed_context: a_signed_context.clone(),
        bob_signed_context: b_signed_context.clone(),
    };

    let clear_config_2 = generate_clear_config(&vault_id, &vault_id);
    let clear_2 = ClearCall {
        alice: order_bob.to_owned(),
        bob: order_alice.to_owned(),
        clear_config: clear_config_2,
        alice_signed_context: b_signed_context,
        bob_signed_context: a_signed_context,
    };

    let clear_configs = vec![clear_1, clear_2];

    let multi_clear_bytes = generate_multi_clear(&clear_configs);

    let multicall_func = orderbook
        .connect(&bounty_bot)
        .await
        .multicall(multi_clear_bytes);

    let tx_multicall = multicall_func.send().await?;

    // Tx hash that hold all the logs
    let clears_tx_hash = tx_multicall.tx_hash();

    let clear_events = get_clear_events(&orderbook, &clears_tx_hash).await?;
    let after_clear_events = get_after_clear_events(&orderbook, &clears_tx_hash).await?;

    // It should emit the same amount of events both parts
    assert_eq!(clear_events.len(), after_clear_events.len());

    // Wait for Subgraph sync
    wait().await?;

    for (index, after_clear) in after_clear_events.iter().enumerate() {
        let order_clear_id = format!("{:?}-{}", clears_tx_hash, index);
        let get_order_clear_state_change_entity_id = order_clear_id.clone();

        let clear_state_change = &after_clear.clear_state_change;

        let resp = Query::order_clear_state_change(&get_order_clear_state_change_entity_id).await?;

        assert_eq!(resp.order_clear, order_clear_id);

        assert_eq!(resp.a_output, clear_state_change.alice_output);
        assert_eq!(resp.b_output, clear_state_change.bob_output);

        assert_eq!(resp.a_input, clear_state_change.alice_input);
        assert_eq!(resp.b_input, clear_state_change.bob_input);
    }
    Ok(())
}

#[tokio::main]
#[test]
async fn token_vault_take_order_entity_take_order_test() -> anyhow::Result<()> {
    let orderbook = get_orderbook().await?;

    let alice = get_wallet(1);
    let bob = get_wallet(2);

    // Connect the orderbook to another wallet
    let orderbook = orderbook.connect(&alice).await;

    // Vault id
    let vault_id = generate_random_u256();

    // Deploy ExpressionDeployerNP for the config
    let expression_deployer = get_expression_deployer().await?;
    // let expression_deployer = get_expression_deployer().await?;

    // Deploy ERC20 token contract (A) connected to Alice
    let token_input = deploy_erc20_mock(None).await?;

    // Deploy ERC20 token contract (B) connected to Alice
    let token_output = deploy_erc20_mock(None).await?;

    // Build OrderConfig
    let order_config = generate_order_config(
        expression_deployer,
        &token_input,
        Some(vault_id),
        &token_output,
        Some(vault_id),
    )
    .await;

    // Add the order
    let add_order_func = orderbook.add_order(order_config.clone());
    let tx_add_order = add_order_func.send().await?;

    // Decode events from the transaction
    let add_order_data = get_add_order_event(&orderbook, &tx_add_order).await?;

    // Amount to deposit
    let amount_b = get_amount_tokens(1000, token_output.decimals().call().await.unwrap());

    // Fill to Alice with tokens
    mint_tokens(&amount_b, &alice.address(), &token_output).await?;

    // Connect token to Alice and approve Orderbook to move tokens
    approve_tokens(
        &amount_b,
        &orderbook.address(),
        &token_output.connect(&alice).await,
    )
    .await?;

    // Alice deposit tokens
    let deposit_func = orderbook.deposit(token_output.address(), vault_id, amount_b);
    let _ = deposit_func.send().await?;

    // BOB TAKE THE ORDER

    // Take Order configs
    let minimum_input = U256::from(0);
    let maximum_input = get_amount_tokens(1000, token_output.decimals().call().await.unwrap());
    let maximum_io_ratio = U256::from(10000000000000000000u64); // 10e18

    let take_order_config = TakeOrderConfig {
        order: add_order_data.order,
        input_io_index: U256::zero(),
        output_io_index: U256::zero(),
        signed_context: Vec::new(),
    };

    let take_orders_config = TakeOrdersConfigV2 {
        minimum_input,
        maximum_input,
        maximum_io_ratio,
        orders: vec![take_order_config],
        data: Bytes::new(), // Empty data
    };

    // Fill bob with token A (token input of the order)
    // let amount_a = get_amount_tokens(1000, token_a.decimals().call().await.unwrap());
    let amount_a = amount_b
        .saturating_mul(maximum_io_ratio)
        .checked_div(U256::from(1000000000000000000u64)) //  1e18
        .unwrap();

    mint_tokens(&amount_a, &bob.address(), &token_input).await?;

    // Connect token to Bob and approve Orderbook to move tokens
    approve_tokens(
        &amount_a,
        &orderbook.address(),
        &token_input.connect(&bob).await,
    )
    .await?;

    // Take the order
    let take_order_func = orderbook
        .connect(&bob)
        .await
        .take_orders(take_orders_config);
    let tx_take_order = take_order_func.send().await?;

    let tx_receipt = tx_take_order.await?.unwrap();

    let take_order_tx_hash = &tx_receipt.transaction_hash;

    let take_order_events = get_take_order_events(&orderbook, &take_order_tx_hash).await?;

    // Just one take order happned in this transaction
    assert!(take_order_events.len() == 1);

    // Using index 0 since only one take order was made in this tx
    let take_order_entity = format!("{:?}-{}", take_order_tx_hash, 0);
    let token_vault_input = format!(
        "{}-{:?}-{:?}",
        vault_id,
        alice.address(),
        token_input.address()
    );
    let token_vault_output = format!(
        "{}-{:?}-{:?}",
        vault_id,
        alice.address(),
        token_output.address()
    );

    let token_vault_take_order_a_id = format!("{}-{}", take_order_entity, token_vault_input);
    let token_vault_take_order_b_id = format!("{}-{}", take_order_entity, token_vault_output);

    let resp_a = Query::token_vault_take_order(&token_vault_take_order_a_id).await?;
    let resp_b = Query::token_vault_take_order(&token_vault_take_order_b_id).await?;

    // Token Input
    assert!(resp_a.was_input);
    assert!(!resp_a.was_output);
    assert_eq!(resp_a.take_order, take_order_entity);
    assert_eq!(resp_a.token_vault, token_vault_input);

    // Token Output
    assert!(!resp_b.was_input);
    assert!(resp_b.was_output);
    assert_eq!(resp_b.take_order, take_order_entity);
    assert_eq!(resp_b.token_vault, token_vault_output);

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
