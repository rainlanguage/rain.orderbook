use std::collections::HashMap;

use crate::{error::CommandResult, shared_state::SharedState};
use rain_orderbook_common::fuzz::*;
use tauri::State;

#[tauri::command]
pub async fn make_charts(dotrain: String) -> CommandResult<ChartData> {
    let runner = FuzzRunner::new(None);
    Ok(runner.make_chart_data(&dotrain, None, None).await?)
}

#[tauri::command]
pub async fn make_deployment_debug(
    dotrain: String,
    settings: Option<String>,
    block_numbers: Option<HashMap<u64, u64>>,
    shared_state: State<'_, SharedState>,
) -> CommandResult<DeploymentsDebugDataMap> {
    let mut runner = shared_state.debug_runner.lock().await;
    Ok(runner
        .make_debug_data(&dotrain, settings, None, block_numbers)
        .await?)
}
