use std::sync::Arc;

use crate::{
    generated::{
        AddOrderCall, ClearCall, ClearConfig, DepositCall, ERC20Mock, EvaluableConfigV2, Io,
        OrderConfigV2, RainterpreterExpressionDeployer, SignedContextV1, WithdrawCall,
    },
    utils::{generate_random_u256, mock_rain_doc},
};
use ethers::{
    core::{abi::AbiEncode, k256::ecdsa::SigningKey},
    prelude::SignerMiddleware,
    providers::{Http, Middleware, PendingTransaction, Provider},
    signers::{Signer, Wallet},
    types::{Address, Block, Bytes, TxHash, H256, U256},
};

use super::{get_provider, get_wallet, hash_keccak};

/// A Deposit configuration struct to encode deposit to be used with multicall
pub struct TestDepositConfig {
    pub token: Address,
    pub vault_id: U256,
    pub amount: U256,
}

/// A Withdraw configuration struct to encode withdraw to be used with multicall
pub struct TestWithdrawConfig {
    pub token: Address,
    pub vault_id: U256,
    pub target_amount: U256,
}

/// Hold all the index from Context inside Orderbook and emitted by Context event.
///
/// If the length of the Context array is above this enum, they are signed context.
pub enum ContextIndex {
    BaseContext = 0,
    CallingContextColumn = 1,
    CalculationsColumn = 2,
    VaultInputsColumn = 3,
    VaultOutputsColumn = 4,
}

impl ContextIndex {
    pub fn from_usize(value: usize) -> Option<Self> {
        if Self::BaseContext as usize == value {
            return Some(Self::BaseContext);
        }

        if Self::CallingContextColumn as usize == value {
            return Some(Self::CallingContextColumn);
        }

        if Self::CalculationsColumn as usize == value {
            return Some(Self::CalculationsColumn);
        }

        if Self::VaultInputsColumn as usize == value {
            return Some(Self::VaultInputsColumn);
        }

        if Self::VaultOutputsColumn as usize == value {
            return Some(Self::VaultOutputsColumn);
        }

        None
    }
}

pub async fn get_block_data(tx_hash: &TxHash) -> anyhow::Result<Block<H256>> {
    let provider = get_provider().await?;

    let pending_tx = PendingTransaction::new(*tx_hash, provider);

    let receipt = match pending_tx.await? {
        Some(receipt) => receipt,
        None => return Err(anyhow::Error::msg("receipt not found")),
    };

    let block_number = match receipt.block_number {
        Some(block_number) => block_number,
        None => return Err(anyhow::Error::msg("block number not found")),
    };

    match provider.get_block(block_number).await? {
        Some(block_data) => Ok(block_data),
        None => return Err(anyhow::Error::msg("block data not found")),
    }
}

pub async fn mint_tokens(
    amount: &U256,
    target: &Address,
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> anyhow::Result<()> {
    token.mint(*target, *amount).send().await?.await?;

    Ok(())
}

pub async fn approve_tokens(
    amount: &U256,
    spender: &Address,
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> anyhow::Result<()> {
    token.approve(*spender, *amount).send().await?.await?;

    Ok(())
}

pub async fn get_decimals(address: Address) -> anyhow::Result<u8> {
    let wallet = get_wallet(0);
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.clone().with_chain_id(chain_id.as_u64()),
    ));

    let contract = ERC20Mock::new(address, client);

    Ok(contract.decimals().call().await?)
}

pub async fn generate_order_config(
    expression_deployer: &RainterpreterExpressionDeployer<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    >,
    token_input: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id_input: Option<U256>,
    token_output: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id_output: Option<U256>,
) -> OrderConfigV2 {
    let io_input = generate_io(token_input, vault_id_input).await;
    let io_output = generate_io(token_output, vault_id_output).await;

    let eval_config = generate_eval_config(expression_deployer).await;

    // Build the OrderConfig and return it
    OrderConfigV2 {
        valid_inputs: vec![io_input],
        valid_outputs: vec![io_output],
        evaluable_config: eval_config,
        meta: mock_rain_doc(),
    }
}

async fn generate_io(
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id: Option<U256>,
) -> Io {
    // Build the IO and return it
    Io {
        token: token.address(),
        decimals: token.decimals().await.unwrap(),
        vault_id: vault_id.unwrap_or(generate_random_u256()),
    }
}

async fn generate_eval_config(
    expression_deployer: &RainterpreterExpressionDeployer<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    >,
) -> EvaluableConfigV2 {
    // let data_parse = Bytes::from_static(b"_ _ _:block-timestamp() chain-id() block-number();:;");
    let data_parse =
        Bytes::from_static(b"_ _ _:block-timestamp() 6000000000000000000 1000000000000000000;:;");
    // Bytes::from_static(b"_ _ _:block-timestamp() 6000000000000000000 1000000000000000000;:;");
    let (bytecode, constants) = expression_deployer
        .parse(data_parse.clone())
        .await
        .expect("cannot get value from parse");

    // Build the EvaluableConfig and return it
    EvaluableConfigV2 {
        deployer: expression_deployer.address(),
        bytecode,
        constants,
    }
}

/// From given orders, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_add_order(orders: Vec<&OrderConfigV2>) -> Vec<Bytes> {
    let mut data: Vec<Bytes> = Vec::new();

    for order in orders {
        let call_config = AddOrderCall {
            config: order.to_owned(),
        };

        let encoded_call = Bytes::from(AbiEncode::encode(call_config));

        // Push the bytes
        data.push(encoded_call);
    }

    return data;
}

/// From given arguments, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_deposit(deposit_configs: &Vec<TestDepositConfig>) -> Vec<Bytes> {
    let mut data: Vec<Bytes> = Vec::new();

    for config in deposit_configs {
        let call_config = DepositCall {
            token: config.token,
            vault_id: config.vault_id,
            amount: config.amount,
        };

        let encoded_call = Bytes::from(AbiEncode::encode(call_config));

        // Push the bytes
        data.push(encoded_call);
    }

    return data;
}

/// From given arguments, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_withdraw(configs: &Vec<TestWithdrawConfig>) -> Vec<Bytes> {
    let mut data: Vec<Bytes> = Vec::new();

    for config in configs {
        let call_config = WithdrawCall {
            token: config.token,
            vault_id: config.vault_id,
            target_amount: config.target_amount,
        };

        let encoded_call = Bytes::from(AbiEncode::encode(call_config));

        // Push the bytes
        data.push(encoded_call);
    }

    return data;
}

/// The function assume that all the IO index are zero.
pub fn generate_clear_config(
    alice_bounty_vault_id: &U256,
    bob_bounty_vault_id: &U256,
) -> ClearConfig {
    ClearConfig {
        alice_input_io_index: U256::zero(),
        alice_output_io_index: U256::zero(),
        bob_input_io_index: U256::zero(),
        bob_output_io_index: U256::zero(),
        alice_bounty_vault_id: *alice_bounty_vault_id,
        bob_bounty_vault_id: *bob_bounty_vault_id,
    }
}

/// From given arguments, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_clear(configs: &Vec<ClearCall>) -> Vec<Bytes> {
    let mut data: Vec<Bytes> = Vec::new();

    for config in configs {
        let encoded_call = Bytes::from(AbiEncode::encode(config.to_owned()));

        // Push the bytes
        data.push(encoded_call);
    }

    return data;
}

pub async fn generate_signed_context_v1(
    wallet: &Wallet<SigningKey>,
) -> anyhow::Result<SignedContextV1> {
    let context = vec![
        generate_random_u256(),
        generate_random_u256(),
        generate_random_u256(),
    ];

    // Removing the first 64bits (2bytes) that include the tuple mark and the array length
    let encoded: Vec<u8> = AbiEncode::encode(context.clone())
        .into_iter()
        .skip(64)
        .collect();

    let hash = hash_keccak(&Bytes::from(encoded).to_vec())
        .as_bytes()
        .to_vec();

    let signed_message = wallet.sign_message(hash.clone()).await?;

    let signature = Bytes::from(signed_message.to_vec());

    Ok(SignedContextV1 {
        signer: wallet.address(),
        context,
        signature,
    })
}
