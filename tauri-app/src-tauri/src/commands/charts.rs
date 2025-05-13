use crate::{error::CommandResult, shared_state::SharedState};
use rain_orderbook_common::fuzz::*;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn make_charts(dotrain: String) -> CommandResult<ChartData> {
    let runner = FuzzRunner::new(None);
    let mut context = FuzzRunnerContext::new(&dotrain, None, None)?;
    Ok(runner.make_chart_data(&mut context).await?)
}

#[tauri::command]
pub async fn make_deployment_debug(
    dotrain: String,
    settings: Option<String>,
    block_numbers: Option<HashMap<u64, u64>>,
    shared_state: State<'_, SharedState>,
) -> CommandResult<DeploymentsDebugDataMap> {
    let mut runner = shared_state.debug_runner.lock().await;
    let mut context = FuzzRunnerContext::new(&dotrain, settings, None)?;
    Ok(runner.make_debug_data(&mut context, block_numbers).await?)
}
