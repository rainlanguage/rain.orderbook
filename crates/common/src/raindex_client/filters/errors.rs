#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentFilterStoreError {
    LoadError(String),
    SaveError(String),
}

impl std::fmt::Display for PersistentFilterStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistentFilterStoreError::LoadError(msg) => write!(f, "Load error: {}", msg),
            PersistentFilterStoreError::SaveError(msg) => write!(f, "Save error: {}", msg),
        }
    }
}

impl std::error::Error for PersistentFilterStoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterError {
    InvalidUrlParams(String),
}

impl std::fmt::Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterError::InvalidUrlParams(msg) => write!(f, "Invalid URL parameters: {}", msg),
        }
    }
}

impl std::error::Error for FilterError {}
