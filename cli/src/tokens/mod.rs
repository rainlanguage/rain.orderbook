use crate::{
    cli::registry::IERC20,
    gasoracle::{gas_price_oracle, is_block_native_supported},
};
use anyhow::anyhow;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Bytes, Eip1559TransactionRequest, H160, U256, U64},
    utils::parse_units,
};
use std::{convert::TryFrom, sync::Arc};
use tracing::error;

/// Builds and returns [Eip1559TransactionRequest] instance for token approval.
/// The integrity of the transaction data is ensured, provided that the input parameters are valid.
/// The transaction can then be submitted to the blockchain via any valid signer.
///
/// # Arguments
/// * `token_address` - Address of the ERC20 token to be approved.
/// * `token_amount` - Fully denominated amount of tokens to be approved.
/// * `approver_address` - Address of the entity to be approved.
/// * `rpc_url` - Provider RPC URL.
/// * `signer_address` - Address of the signer.
/// * `blocknative_api_key` - Optional Blocknative API key.
pub async fn approve_tokens(
    token_address: H160,
    token_amount: U256,
    approver_address: H160,
    rpc_url: String,
    signer_address: H160,
    blocknative_api_key: Option<String>,
) -> anyhow::Result<Eip1559TransactionRequest> {
    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(anyhow!(err));
        }
    };

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let token_contract = IERC20::new(token_address.clone(), Arc::new(provider.clone()));
    let token_balance: U256 = token_contract
        .balance_of(signer_address)
        .call()
        .await
        .unwrap();

    if token_balance.gt(&token_amount.clone()) {
        let approve_tx = token_contract.approve(approver_address.clone(), token_amount.clone());
        let data: Bytes = approve_tx.calldata().unwrap();

        let mut approve_tx = Eip1559TransactionRequest::new();
        approve_tx.to = Some(token_address.into());
        approve_tx.value = Some(U256::zero());
        approve_tx.data = Some(data);
        approve_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap());

        if is_block_native_supported(chain_id) {
            let (max_priority, max_fee) = gas_price_oracle(blocknative_api_key, chain_id)
                .await
                .unwrap();
            let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into();
            let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into();

            approve_tx.max_priority_fee_per_gas = Some(max_priority);
            approve_tx.max_fee_per_gas = Some(max_fee);
        }

        Ok(approve_tx)
    } else {
        error!("INSUFFICIENT BALANCE");
        return Err(anyhow!("INSUFFICIENT BALANCE"));
    }
}
