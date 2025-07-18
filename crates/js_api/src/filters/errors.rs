#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentFilterStoreError {
    LoadError(String),
    SaveError(String),
    LocalStorageInitError(String),
    LocalStorageUnavailable,
    WindowNotAvailable,
    FilterUpdateError(String),
}

impl std::fmt::Display for PersistentFilterStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistentFilterStoreError::LoadError(msg) => write!(f, "Load error: {}", msg),
            PersistentFilterStoreError::SaveError(msg) => write!(f, "Save error: {}", msg),
            PersistentFilterStoreError::LocalStorageUnavailable => {
                write!(f, "Local storage is unavailable")
            }
            PersistentFilterStoreError::WindowNotAvailable => write!(f, "Window is not available"),
            PersistentFilterStoreError::LocalStorageInitError(msg) => {
                write!(f, "Local storage initialization error: {}", msg)
            }
            PersistentFilterStoreError::FilterUpdateError(msg) => {
                write!(f, "Filter update error: {}", msg)
            }
        }
    }
}

impl std::error::Error for PersistentFilterStoreError {}

impl From<anyhow::Error> for PersistentFilterStoreError {
    fn from(err: anyhow::Error) -> Self {
        PersistentFilterStoreError::FilterUpdateError(format!("{:?}", err))
    }
}

#[cfg(target_family = "wasm")]
impl From<PersistentFilterStoreError> for wasm_bindgen_utils::result::WasmEncodedError {
    fn from(err: PersistentFilterStoreError) -> Self {
        wasm_bindgen_utils::result::WasmEncodedError {
            msg: err.to_string(),
            readable_msg: err.to_string(),
        }
    }
}
