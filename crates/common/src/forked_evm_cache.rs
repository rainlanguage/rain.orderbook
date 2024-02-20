use crate::abi_error::{decode_abi_error, AbiDecodeFailedError, AbiDecodedErrorType};
use alloy_ethers_typecast::transaction::ReadableClientError;
use alloy_primitives::{bytes::Bytes, Address};
use forker::ForkedEvm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, MutexGuard, PoisonError};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum ForkCallError {
    #[error("{0}")]
    CallError(String),
    #[error("{0}")]
    CallReverted(AbiDecodedErrorType),
    #[error("AbiDecodeFailed error: {0}")]
    AbiDecodeFailed(AbiDecodeFailedError),
    #[error("Fork Cache Poisoned")]
    ForkCachePoisoned,
    #[error("Missing expected cache key {0}")]
    ForkCacheKeyMissing(String),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
}

impl From<AbiDecodeFailedError> for ForkCallError {
    fn from(value: AbiDecodeFailedError) -> Self {
        Self::AbiDecodeFailed(value)
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>> for ForkCallError {
    fn from(_value: PoisonError<MutexGuard<'a, HashMap<String, ForkedEvm>>>) -> Self {
        Self::ForkCachePoisoned
    }
}

pub static FORKED_EVM_CACHE: Lazy<ForkedEvmCache> = Lazy::new(ForkedEvmCache::new);

pub struct ForkedEvmCache {
    cache: Arc<Mutex<HashMap<String, ForkedEvm>>>,
}

impl ForkedEvmCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Construct cache key from fork url + block number
    fn make_cache_key(rpc_url: &str, block_number: u64) -> String {
        format!("{}-{}", rpc_url, block_number)
    }

    /// Create a new evm fork at the provided block number and save to cache
    pub async fn create(&self, rpc_url: &str, block_number: u64) -> Result<(), ForkCallError> {
        let cache_key = Self::make_cache_key(rpc_url, block_number);
        let forked_evm = ForkedEvm::new(rpc_url, Some(block_number), Some(200000u64), None).await;

        let mut forks = self.cache.lock().await;
        forks.insert(cache_key.clone(), forked_evm);

        Ok(())
    }

    /// Get ForkedEvm from cache or create one save to cache if not found.
    pub async fn get_or_create(
        &self,
        rpc_url: &str,
        block_number: u64,
    ) -> Result<String, ForkCallError> {
        let cache_key = Self::make_cache_key(rpc_url, block_number);

        let is_cached = self.cache.lock().await.contains_key(&cache_key);
        if !is_cached {
            self.create(rpc_url, block_number).await?;
        }

        Ok(cache_key)
    }

    /// Call a contract on the cached evm fork
    pub async fn call(
        &self,
        cache_key: String,
        sender: Address,
        address: Address,
        calldata: &[u8],
    ) -> Result<Bytes, ForkCallError> {
        // call contract on evm fork
        let mut cache = self.cache.lock().await;
        let forked_evm = cache
            .get_mut(&cache_key)
            .ok_or(ForkCallError::ForkCacheKeyMissing(cache_key.clone()))?;
        let result = forked_evm
            .call(sender.as_slice(), address.as_slice(), calldata)
            .map_err(|e| ForkCallError::CallError(e.to_string()))?;

        if result.reverted {
            // decode result bytes to error selectors if it was a revert
            Err(ForkCallError::CallReverted(
                decode_abi_error(&result.result).await?,
            ))
        } else {
            Ok(Bytes::from(result.result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::decode;
    use alloy_sol_types::SolValue;

    const FORK_URL: &str = "https://rpc.ankr.com/polygon_mumbai";
    const FORK_BLOCK_NUMBER: u64 = 45122616;
    const FROM_ADDRESS: Address = Address::repeat_byte(0x1);
    const DEPLOY_EXPRESSION_2_SELECTOR: &str = "0xb7f14403"; // deployExpression2(bytes,uint256[])
    const PARSE_SELECTOR: &str = "0xfab4087a"; // parse()

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_fail_parse() {
        // has no semi at the end
        let rainlang_text = r"_: int-add(1)";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text to build calldata
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();

        // this is calling parse() that will not run integrity checks
        // in order to run integrity checks another call should be done on
        // expressionDeployer2() of deployer contract with same process
        let cache_key = FORKED_EVM_CACHE
            .get_or_create(FORK_URL, FORK_BLOCK_NUMBER)
            .await
            .unwrap();
        let result = FORKED_EVM_CACHE
            .call(cache_key, FROM_ADDRESS, parser_address, &calldata)
            .await;
        let expected = AbiDecodedErrorType::Known {
            name: "MissingFinalSemi".to_owned(),
            args: vec!["Uint(0x000000000000000000000000000000000000000000000000000000000000000d_U256, 256)".to_owned()],
            sig: "MissingFinalSemi(uint256)".to_owned(),
        };
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), expected.to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_fail_integrity() {
        // fixed semi error, but still has bad input problem
        // get expressionconfig and call deployer to get integrity checks error
        let rainlang_text = r"_: int-add(1);";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();

        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text
        let cache_key = FORKED_EVM_CACHE
            .get_or_create(FORK_URL, FORK_BLOCK_NUMBER)
            .await
            .unwrap();
        let expression_config = FORKED_EVM_CACHE
            .call(cache_key, FROM_ADDRESS, parser_address, &calldata)
            .await
            .unwrap();

        let mut calldata = decode(DEPLOY_EXPRESSION_2_SELECTOR).unwrap();
        calldata.extend_from_slice(&expression_config); // extend with result of parse() which is expressionConfig
        let deployer_address: Address = "0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b"
            .parse::<Address>()
            .unwrap();

        // get integrity check results, if ends with error, decode with the selectors
        let cache_key = FORKED_EVM_CACHE
            .get_or_create(FORK_URL, FORK_BLOCK_NUMBER)
            .await
            .unwrap();
        let result = FORKED_EVM_CACHE
            .call(cache_key, FROM_ADDRESS, deployer_address, &calldata)
            .await;
        let expected = AbiDecodedErrorType::Known {
            name: "BadOpInputsLength".to_owned(),
            args: vec![
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256)".to_owned(), 
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000002_U256, 256)".to_owned(), 
                "Uint(0x0000000000000000000000000000000000000000000000000000000000000001_U256, 256)".to_owned()
            ],
            sig: "BadOpInputsLength(uint256,uint256,uint256)".to_owned(),
        };
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), expected.to_string());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fork_call_parse_success() {
        // get expressionconfig and call deployer to get integrity checks error
        let rainlang_text = r"_: int-add(1 2);";
        let mut calldata = decode(PARSE_SELECTOR).unwrap();
        calldata.extend_from_slice(&rainlang_text.abi_encode()); // extend with rainlang text
        let parser_address: Address = "0xea3b12393D2EFc4F3E15D41b30b3d020610B9e02"
            .parse::<Address>()
            .unwrap();
        let cache_key = FORKED_EVM_CACHE
            .get_or_create(FORK_URL, FORK_BLOCK_NUMBER)
            .await
            .unwrap();
        let expression_config = FORKED_EVM_CACHE
            .call(cache_key, FROM_ADDRESS, parser_address, &calldata)
            .await
            .unwrap();

        let mut calldata = decode(DEPLOY_EXPRESSION_2_SELECTOR).unwrap();
        calldata.extend_from_slice(&expression_config); // extend with result of parse() which is expressionConfig
        let deployer_address: Address = "0x5155cE66E704c5Ce79a0c6a1b79113a6033a999b"
            .parse::<Address>()
            .unwrap();

        // expression deploys ok so the expressionConfig in previous step can be used to deploy onchain
        let cache_key = FORKED_EVM_CACHE
            .get_or_create(FORK_URL, FORK_BLOCK_NUMBER)
            .await
            .unwrap();
        let result = FORKED_EVM_CACHE
            .call(cache_key, FROM_ADDRESS, deployer_address, &calldata)
            .await
            .unwrap();
        let expected = Bytes::from(
            alloy_primitives::hex::decode(
                "0x000000000000000000000000f22cda7695125487993110d706f3b001c8d106400000000000000000000000008a326d777bc34ea563bd21854b436d458112185b00000000000000000000000064c9b10e815a089698521b10be95c8c9c2ed0b3c000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000020001000000000000000000000000000000000000000000000000000000000000"
            ).unwrap()
        );
        assert_eq!(result, expected);
    }
}
