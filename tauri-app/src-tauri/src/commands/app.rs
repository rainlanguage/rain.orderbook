use rain_orderbook_common::GH_COMMIT_SHA;

use crate::error::CommandResult;

#[tauri::command]
pub async fn get_app_commit_sha() -> CommandResult<String> {
    Ok(GH_COMMIT_SHA.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_app_commit_sha() {
        let commit_sha = get_app_commit_sha().await.unwrap();
        assert_eq!(commit_sha, GH_COMMIT_SHA.to_string());
    }
}
