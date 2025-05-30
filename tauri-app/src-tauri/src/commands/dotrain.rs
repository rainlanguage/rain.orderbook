use crate::error::CommandResult;
use alloy::primitives::{bytes::Bytes, Address};
use rain_orderbook_common::rainlang::parse_rainlang_on_fork;

#[tauri::command]
pub async fn parse_dotrain(
    rainlang: &str,
    rpcs: Vec<String>,
    block_number: u64,
    deployer: Address,
) -> CommandResult<Bytes> {
    Ok(parse_rainlang_on_fork(rainlang, &rpcs, Some(block_number), deployer).await?)
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex::{encode_prefixed, FromHex},
        providers::Provider,
    };

    use super::*;
    use crate::error::CommandError;
    use rain_error_decoding::AbiDecodedErrorType;
    use rain_interpreter_eval::error::ForkCallError;
    use rain_orderbook_common::rainlang::ForkParseError;
    use rain_orderbook_test_fixtures::LocalEvm;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_parse_dotrain_ok() {
        let local_evm = LocalEvm::new().await;
        let deployer = *local_evm.deployer.address();
        let rpc_url = local_evm.url();
        let block_number = local_evm.provider.get_block_number().await.unwrap();

        let rainlang = "
/* 0. calculate-io */
_ _: 0 0;

/* 1. handle-io */
:;
        ";

        let bytes = parse_dotrain(rainlang, vec![rpc_url], block_number, deployer)
            .await
            .unwrap();

        let expected = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000075000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100000000000000000000000000000000000";

        assert_eq!(encode_prefixed(&bytes), expected);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_dotrain_err_invalid_rainlang() {
        let local_evm = LocalEvm::new().await;
        let deployer = *local_evm.deployer.address();
        let rpc_url = local_evm.url();
        let block_number = local_evm.provider.get_block_number().await.unwrap();

        let invalid_rainlang = "invalid";
        let err = parse_dotrain(
            invalid_rainlang,
            vec![rpc_url.clone()],
            block_number,
            deployer,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            CommandError::ForkParseError(ForkParseError::ForkCallReverted(
                AbiDecodedErrorType::Known { name, .. }
            )) if name == "MissingFinalSemi"
        ));

        let invalid_rainlang = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
";

        let err = parse_dotrain(invalid_rainlang, vec![rpc_url], block_number, deployer)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::ForkParseError(ForkParseError::ForkCallReverted(
                AbiDecodedErrorType::Known { name, .. }
            )) if name == "UnexpectedLHSChar"
        ));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_dotrain_err_invalid_deployer() {
        let local_evm = LocalEvm::new().await;
        let deployer = *local_evm.deployer.address();
        // Make the deployer address invalid by flipping a bit in the original address
        let deployer = deployer
            .bit_xor(Address::from_hex("0000000000000000000000000000000000000001").unwrap());

        let rpc_url = local_evm.url();
        let block_number = local_evm.provider.get_block_number().await.unwrap();

        let rainlang = "
/* 0. calculate-io */
_ _: 0 0;

/* 1. handle-io */
:;
        ";

        let err = parse_dotrain(rainlang, vec![rpc_url], block_number, deployer)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            CommandError::ForkParseError(ForkParseError::ForkerError(ForkCallError::TypedError(msg)))
            if msg.contains("parse2Call")
        ));
    }
}
