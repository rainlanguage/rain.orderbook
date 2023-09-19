use crate::{
    cli::registry::IOrderBookV3,
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

/// Builds and returns [Eip1559TransactionRequest] instance for `withdrawing tokens from OrderBook`.
/// The integrity of the transaction data is ensured, provided that the input parameters are valid.
/// The transaction can then be submitted to the blockchain via any valid signer.
///
/// # Arguments
/// * `withdraw_token_address` - Address of the ERC20 token to be withdrawn.
/// * `withdraw_token_amount` - Fully denominated amount of tokens to be withdrawn.
/// * `withdraw_vault_id` - Vault ID of the vault to be withdrawn from.
/// * `orderbook_address` - Address of the OrderBook contract.
/// * `rpc_url` - Provider RPC URL.
/// * `signer_address` - Address of the signer associated with vault.
/// * `blocknative_api_key` - Optional Blocknative API key.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
/// use rain_cli_ob::orderbook::withdraw::v3::withdraw_tokens;
/// use ethers::types::{U256, H160, Eip1559TransactionRequest};
///
/// async fn withdraw() {
///     let rpc_url = "https://polygon.llamarpc.com/".to_string() ;
///     let orderbook_address = H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap() ;  
///     let withdraw_token_address = H160::from_str(&String::from("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")).unwrap() ;
///     let withdraw_token_amount = U256::from_dec_str("1000000").unwrap();
///     let withdraw_vault_id = U256::from_dec_str("123456789").unwrap();
///     let signer_address = H160::from_str(&String::from("0x53AB61eE41FA202227Eb4e7B176208FC626DC8A9")).unwrap() ;
///
///     let _ = withdraw_tokens(
///         withdraw_token_address,
///         withdraw_token_amount,
///         withdraw_vault_id,
///         orderbook_address,
///         rpc_url,
///         signer_address,
///         None
///     ).await.unwrap() ;
///     
/// }
/// ```
pub async fn withdraw_tokens(
    withdraw_token_address: H160,
    withdraw_token_amount: U256,
    wihtdraw_vault_id: U256,
    orderbook_address: H160,
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

    let orderbook = IOrderBookV3::new(orderbook_address, Arc::new(provider.clone()));

    let vault_balance: U256 = orderbook
        .vault_balance(signer_address, withdraw_token_address, wihtdraw_vault_id)
        .call()
        .await
        .unwrap();

    if withdraw_token_amount.gt(&vault_balance) {
        error!("INSUFFICIENT VAULT BALANCE FOR WITHDRAWAL");
        return Err(anyhow!("INSUFFICIENT VAULT BALANCE FOR WITHDRAWAL"));
    }

    let withdraw_tx = orderbook.withdraw(
        withdraw_token_address,
        wihtdraw_vault_id,
        withdraw_token_amount,
    );
    let withdraw_data: Bytes = withdraw_tx.calldata().unwrap();

    let mut withdraw_tx = Eip1559TransactionRequest::new();
    withdraw_tx.to = Some(orderbook_address.into());
    withdraw_tx.value = Some(U256::zero());
    withdraw_tx.data = Some(withdraw_data);
    withdraw_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap());

    if is_block_native_supported(chain_id) {
        let (max_priority, max_fee) = gas_price_oracle(blocknative_api_key, chain_id)
            .await
            .unwrap();
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into();
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into();

        withdraw_tx.max_priority_fee_per_gas = Some(max_priority);
        withdraw_tx.max_fee_per_gas = Some(max_fee);
    }
    Ok(withdraw_tx)
}

#[cfg(test)]
mod test {
    use crate::orderbook::withdraw::v3::withdraw_tokens;
    use ethers::types::{H160, U256};
    use std::str::FromStr;

    #[tokio::test]
    #[should_panic]
    pub async fn test_withdraw() {
        let rpc_url = "https://polygon.llamarpc.com".to_string();
        let orderbook_address =
            H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap();
        let withdraw_token_address = H160::random();
        let withdraw_token_amount = U256::from(H160::random().as_bytes());
        let withdraw_vault_id = U256::from(H160::random().as_bytes());
        let signer_address = H160::random();

        let _ = withdraw_tokens(
            withdraw_token_address,
            withdraw_token_amount,
            withdraw_vault_id,
            orderbook_address,
            rpc_url,
            signer_address,
            None,
        )
        .await
        .unwrap();
    }
}
