use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::provider::ReadProvider;
use rain_orderbook_bindings::IOrderBookV5::{takeOrders3Call, TakeOrdersConfigV4};
use rain_orderbook_bindings::IERC20::approveCall;
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
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),
}

pub struct AllowanceCheckResult {
    pub needs_approval: bool,
    pub required_raw: U256,
}

pub async fn check_taker_balance(
    erc20: &crate::erc20::ERC20,
    taker: Address,
    max_sell: Float,
) -> Result<U256, PreflightError> {
    let decimals = erc20.decimals().await?;
    let (max_sell_raw, _) = max_sell.to_fixed_decimal_lossy(decimals)?;

    let balance = erc20.get_account_balance(taker).await?;
    if balance < max_sell_raw {
        return Err(PreflightError::InsufficientBalance {
            balance: balance.to_string(),
            required: max_sell_raw.to_string(),
        });
    }

    Ok(max_sell_raw)
}

pub async fn check_taker_allowance(
    erc20: &crate::erc20::ERC20,
    taker: Address,
    orderbook: Address,
    required_raw: U256,
) -> Result<AllowanceCheckResult, PreflightError> {
    let allowance = erc20.allowance(taker, orderbook).await?;
    Ok(AllowanceCheckResult {
        needs_approval: allowance < required_raw,
        required_raw,
    })
}

pub fn build_approval_calldata(spender: Address, amount: U256) -> Bytes {
    let calldata = approveCall { spender, amount }.abi_encode();
    Bytes::copy_from_slice(&calldata)
}

pub async fn check_taker_balance_and_allowance(
    erc20: &crate::erc20::ERC20,
    taker: Address,
    orderbook: Address,
    max_sell: Float,
) -> Result<AllowanceCheckResult, PreflightError> {
    let required_raw = check_taker_balance(erc20, taker, max_sell).await?;
    check_taker_allowance(erc20, taker, orderbook, required_raw).await
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

    let min_input = config.minimumInput;

    let mut low: usize = 0;
    let mut high: usize = num_orders;
    let mut last_failing: Option<usize> = None;

    while low < high {
        let mid = (low + high) / 2;

        let prefix_config = TakeOrdersConfigV4 {
            minimumInput: min_input,
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
    use rain_orderbook_bindings::IERC20::approveCall;

    #[test]
    fn test_preflight_error_display() {
        let err = PreflightError::InsufficientBalance {
            balance: "100".to_string(),
            required: "200".to_string(),
        };
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("200"));
    }

    #[test]
    fn test_build_approval_calldata() {
        let spender = Address::from([0x11u8; 20]);
        let amount = U256::from(1000u64);

        let calldata = build_approval_calldata(spender, amount);

        let decoded = approveCall::abi_decode(&calldata).expect("Should decode approval calldata");
        assert_eq!(decoded.spender, spender);
        assert_eq!(decoded.amount, amount);
    }

    #[test]
    fn test_allowance_check_result_needs_approval_when_insufficient() {
        let result = AllowanceCheckResult {
            needs_approval: true,
            required_raw: U256::from(1000u64),
        };
        assert!(result.needs_approval);
        assert_eq!(result.required_raw, U256::from(1000u64));
    }

    #[test]
    fn test_allowance_check_result_no_approval_when_sufficient() {
        let result = AllowanceCheckResult {
            needs_approval: false,
            required_raw: U256::from(500u64),
        };
        assert!(!result.needs_approval);
    }
}
