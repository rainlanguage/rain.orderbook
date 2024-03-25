use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use crate::dotrain_add_order_lsp::LANG_SERVICES;
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use alloy_primitives::{bytes::Bytes, Address};
use dotrain::error::ComposeError;
use dotrain::RainDocument;
use dotrain::Rebind;
use once_cell::sync::Lazy;
use rain_interpreter_eval::error::AbiDecodedErrorType;
use rain_interpreter_eval::error::ForkCallError;
use rain_interpreter_eval::eval::ForkParseArgs;
use rain_interpreter_eval::fork::Forker;
use rain_interpreter_eval::fork::NewForkedEvm;
use std::collections::HashMap;
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
    ForkCallReverted(AbiDecodedErrorType),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error("Failed to read Parser address from deployer")]
    ReadParserAddressFailed,
}

impl From<AbiDecodedErrorType> for ForkParseError {
    fn from(value: AbiDecodedErrorType) -> Self {
        Self::ForkCallReverted(value)
    }
}
impl From<ForkCallError> for ForkParseError {
    fn from(value: ForkCallError) -> Self {
        match value {
            ForkCallError::AbiDecodedError(v) => Self::ForkCallReverted(v),
            other => Self::ForkerError(other),
        }
    }
}

/// Arbitrary address used to call from
pub const SENDER_ADDRESS: Address = Address::repeat_byte(0x1);

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

/// Compose to rainlang string
pub fn compose_to_rainlang(
    dotrain: String,
    bindings: HashMap<String, String>,
) -> Result<String, ComposeError> {
    let meta_store = LANG_SERVICES.meta_store();

    let mut rebinds = None;
    if !bindings.is_empty() {
        rebinds = Some(
            bindings
                .iter()
                .map(|(key, value)| Rebind(key.clone(), value.clone()))
                .collect(),
        );
    };
    let rain_document = RainDocument::create(
        dotrain.clone(),
        Some(meta_store.clone()),
        None,
        rebinds.clone(),
    );

    let mut final_bindings: Vec<Rebind> = vec![];

    // search the name space hash map for NamespaceItems that are elided and make a vec of the keys
    let elided_binding_keys = Arc::new(
        rain_document
            .namespace()
            .iter()
            .filter(|(_, v)| v.is_elided_binding())
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>(),
    );

    // for each scenario elided bindings, set 0
    for elided_binding in elided_binding_keys.as_slice() {
        let hex = format!("0x{}", alloy_primitives::hex::encode([0; 32]));
        final_bindings.push(Rebind(elided_binding.to_string(), hex));
    }
    if let Some(main_bindings) = rebinds {
        final_bindings.extend(main_bindings);
    }

    let rain_document = RainDocument::create(dotrain, Some(meta_store), None, Some(final_bindings));

    rain_document.compose(&ORDERBOOK_ORDER_ENTRYPOINTS)
}
