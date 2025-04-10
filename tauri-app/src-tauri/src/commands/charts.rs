use crate::error::CommandResult;
use rain_orderbook_common::fuzz::*;

#[tauri::command]
pub async fn make_charts(dotrain: String) -> CommandResult<ChartData> {
    let fuzzer = FuzzRunner::new(dotrain.as_str(), None, None).await?;

    Ok(fuzzer.make_chart_data().await?)
}

#[tauri::command]
pub async fn make_deployment_debug(
    dotrain: String,
    settings: Option<String>,
    block_number: Option<u64>,
) -> CommandResult<DeploymentDebugData> {
    let fuzzer = FuzzRunner::new(dotrain.as_str(), settings, None).await?;
    Ok(fuzzer.make_debug_data(block_number).await?)
}
