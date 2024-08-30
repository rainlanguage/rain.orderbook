use rain_orderbook_env::GH_COMMIT_SHA;

use crate::error::CommandResult;

#[tauri::command]
pub async fn get_app_commit_sha() -> CommandResult<String> {
    Ok(GH_COMMIT_SHA.to_string())
}
