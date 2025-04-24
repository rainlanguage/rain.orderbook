use crate::{error::CommandResult, shared_state::SharedState};
use rain_orderbook_common::fuzz::*;
use tauri::State;

#[tauri::command]
pub async fn make_charts(
    dotrain: String,
    shared_state: State<'_, SharedState>,
) -> CommandResult<ChartData> {
    let runner = shared_state
        .get_or_create_fuzz_runner(dotrain, None)
        .await?;
    Ok(runner.make_chart_data().await?)
}

#[tauri::command]
pub async fn make_deployment_debug(
    dotrain: String,
    settings: Option<String>,
    block_number: Option<u64>,
    shared_state: State<'_, SharedState>,
) -> CommandResult<DeploymentDebugData> {
    let runner = shared_state
        .get_or_create_fuzz_runner(dotrain, settings)
        .await?;
    Ok(runner.make_debug_data(block_number).await?)
}
