use crate::local_db::{LocalDb, LocalDbError};
use crate::raindex_client::{RaindexClient, RaindexError};
use wasm_bindgen_utils::result::WasmEncodedError;
use wasm_bindgen_utils::{prelude::*, wasm_export};

pub mod executor;
pub mod query;
pub mod sync;

impl From<LocalDbError> for WasmEncodedError {
    fn from(value: LocalDbError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "getLocalDbClient", preserve_js_class)]
    pub fn get_local_db_client(&self, chain_id: u32) -> Result<LocalDb, RaindexError> {
        let rpcs = self.get_rpc_urls_for_chain(chain_id)?;
        LocalDb::new_with_regular_rpcs(rpcs).map_err(RaindexError::from)
    }
}
