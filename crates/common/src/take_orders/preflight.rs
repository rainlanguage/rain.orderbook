use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::provider::ReadProvider;
use rain_orderbook_bindings::IOrderBookV5::{takeOrders3Call, TakeOrdersConfigV4};
use thiserror::Error;

const DEFAULT_GAS_CAP: u64 = 30_000_000;

#[derive(Debug, Error)]
pub enum PreflightError {
    #[error("Float error: {0}")]
    Float(#[from] rain_math_float::FloatError),
    #[error("Provider error: {0}")]
    Provider(#[from] rain_orderbook_bindings::provider::ReadProviderError),
    #[error("ERC20 error: {0}")]
    Erc20(#[from] crate::erc20::Error),
    #[error("Insufficient balance: taker has {balance} but needs {required}")]
    InsufficientBalance { balance: String, required: String },
    #[error("Insufficient allowance: taker approved {allowance} but needs {required}")]
    InsufficientAllowance { allowance: String, required: String },
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),
}

pub async fn check_taker_balance_and_allowance(
    erc20: &crate::erc20::ERC20,
    taker: Address,
    orderbook: Address,
    max_sell: Float,
) -> Result<(), PreflightError> {
    let decimals = erc20.decimals().await?;
    let (max_sell_raw, _) = max_sell.to_fixed_decimal_lossy(decimals)?;

    let balance = erc20.get_account_balance(taker).await?;
    if balance < max_sell_raw {
        return Err(PreflightError::InsufficientBalance {
            balance: balance.to_string(),
            required: max_sell_raw.to_string(),
        });
    }

    let allowance = erc20.allowance(taker, orderbook).await?;
    if allowance < max_sell_raw {
        return Err(PreflightError::InsufficientAllowance {
            allowance: allowance.to_string(),
            required: max_sell_raw.to_string(),
        });
    }

    Ok(())
}

pub async fn simulate_take_orders(
    provider: &ReadProvider,
    orderbook: Address,
    taker: Address,
    config: &TakeOrdersConfigV4,
    block_number: u64,
) -> Result<(), String> {
    let calldata = takeOrders3Call {
        config: config.clone(),
    }
    .abi_encode();

    let tx = TransactionRequest::default()
        .with_from(taker)
        .with_to(orderbook)
        .with_input(Bytes::copy_from_slice(&calldata))
        .with_gas_limit(DEFAULT_GAS_CAP);

    let block_id = alloy::eips::BlockNumberOrTag::Number(block_number).into();

    provider
        .call(tx.into())
        .block(block_id)
        .await
        .map_err(format_rpc_error)?;

    Ok(())
}

pub async fn find_failing_order_index(
    provider: &ReadProvider,
    orderbook: Address,
    taker: Address,
    config: &TakeOrdersConfigV4,
    block_number: u64,
) -> Option<usize> {
    let num_orders = config.orders.len();
    if num_orders == 0 {
        return None;
    }
    if num_orders == 1 {
        return Some(0);
    }

    let zero = Float::zero().ok()?.get_inner();

    let mut low: usize = 0;
    let mut high: usize = num_orders;
    let mut last_failing: Option<usize> = None;

    while low < high {
        let mid = (low + high) / 2;

        let prefix_config = TakeOrdersConfigV4 {
            minimumInput: zero,
            maximumInput: config.maximumInput,
            maximumIORatio: config.maximumIORatio,
            orders: config.orders[..=mid].to_vec(),
            data: config.data.clone(),
        };

        let result =
            simulate_take_orders(provider, orderbook, taker, &prefix_config, block_number).await;

        if result.is_ok() {
            low = mid + 1;
        } else {
            last_failing = Some(mid);
            high = mid;
        }
    }

    last_failing
}

fn format_rpc_error(
    err: alloy::transports::RpcError<alloy::transports::TransportErrorKind>,
) -> String {
    match err {
        alloy::transports::RpcError::ErrorResp(resp) => {
            if let Some(data) = resp.data {
                format!("Revert: {} (data: {})", resp.message, data)
            } else {
                format!("Revert: {}", resp.message)
            }
        }
        other => format!("RPC error: {}", other),
    }
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_error_display() {
        let err = PreflightError::InsufficientBalance {
            balance: "100".to_string(),
            required: "200".to_string(),
        };
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("200"));

        let err = PreflightError::InsufficientAllowance {
            allowance: "50".to_string(),
            required: "100".to_string(),
        };
        assert!(err.to_string().contains("50"));
        assert!(err.to_string().contains("100"));
    }
}
