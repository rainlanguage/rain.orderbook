use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use crate::dotrain_add_order_lsp::LANG_SERVICES;
use dotrain::error::ComposeError;
use dotrain::RainDocument;
use dotrain::Rebind;
use std::collections::HashMap;

/// Compose to rainlang string by setting elided bindings to zero
pub fn compose_to_rainlang(
    dotrain: String,
    bindings: HashMap<String, String>,
) -> Result<String, ComposeError> {
    let meta_store = LANG_SERVICES.meta_store();

    let rebinds = (!bindings.is_empty()).then_some(
        bindings
            .iter()
            .map(|(k, v)| Rebind(k.clone(), v.clone()))
            .collect(),
    );
    let rain_document = RainDocument::create(
        dotrain.clone(),
        Some(meta_store.clone()),
        None,
        rebinds.clone(),
    );

    // search the namespace hash map for NamespaceItems that are elided
    // to set them to 0 and finally merge main bindings into them
    let final_bindings = rain_document
        .namespace()
        .iter()
        .filter_map(|(k, v)| {
            v.is_elided_binding().then_some(Rebind(
                k.clone(),
                alloy_primitives::hex::encode_prefixed([0; 32]),
            ))
        })
        .chain(rebinds.unwrap_or(vec![]))
        .collect::<Vec<Rebind>>();

    // compose a new RainDocument with final injected bindings
    RainDocument::create(dotrain, Some(meta_store), None, Some(final_bindings))
        .compose(&ORDERBOOK_ORDER_ENTRYPOINTS)
}

#[cfg(not(target_family = "wasm"))]
pub use fork_parse::*;

#[cfg(not(target_family = "wasm"))]
mod fork_parse {
    use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
    use alloy_primitives::{bytes::Bytes, Address};
    use once_cell::sync::Lazy;
    use rain_error_decoding::AbiDecodedErrorType;
    use rain_interpreter_eval::error::ForkCallError;
    use rain_interpreter_eval::eval::ForkParseArgs;
    use rain_interpreter_eval::fork::Forker;
    use rain_interpreter_eval::fork::NewForkedEvm;
    use std::sync::Arc;
    use thiserror::Error;
    use tokio::sync::Mutex;

    pub static FORKER: Lazy<Arc<Mutex<Forker>>> = Lazy::new(|| Arc::new(Mutex::new(Forker::new())));

    #[derive(Debug, Error)]
    pub enum ForkParseError {
        #[error("Fork Cache Poisoned")]
        ForkCachePoisoned,
        #[error(transparent)]
        ForkerError(ForkCallError),
        #[error("Fork Call Reverted: {0}")]
        ForkCallReverted(#[from] AbiDecodedErrorType),
        #[error(transparent)]
        ReadableClientError(#[from] ReadableClientError),
        #[error("Failed to read Parser address from deployer")]
        ReadParserAddressFailed,
    }

    impl From<ForkCallError> for ForkParseError {
        fn from(value: ForkCallError) -> Self {
            match value {
                ForkCallError::AbiDecodedError(v) => Self::ForkCallReverted(v),
                other => Self::ForkerError(other),
            }
        }
    }

    /// checks the front matter validity and parses the given rainlang string
    /// with the deployer parsed from the front matter
    /// returns abi encoded expression config on Ok variant
    pub async fn parse_rainlang_on_fork(
        rainlang: &str,
        rpc_url: &str,
        block_number: Option<u64>,
        deployer: Address,
    ) -> Result<Bytes, ForkParseError> {
        // Prepare evm fork
        let block_number_val = match block_number {
            Some(b) => b,
            None => {
                let client = ReadableClientHttp::new_from_url(rpc_url.to_string())?;

                client.get_block_number().await?
            }
        };
        let args = NewForkedEvm {
            fork_url: rpc_url.to_owned(),
            fork_block_number: Some(block_number_val),
        };
        let mut forker = FORKER.lock().await;
        forker.add_or_select(args, None).await?;

        let parse_args = ForkParseArgs {
            rainlang_string: rainlang.to_owned(),
            deployer,
            decode_errors: true,
        };
        let result = forker.fork_parse(parse_args).await?;

        Ok(result.raw.result.0)
    }
}
