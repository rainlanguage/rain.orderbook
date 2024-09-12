use rain_orderbook_common::GH_COMMIT_SHA;

use crate::error::CommandResult;

#[tauri::command]
pub async fn get_app_commit_sha() -> CommandResult<String> {
    Ok(GH_COMMIT_SHA.to_string())
}
