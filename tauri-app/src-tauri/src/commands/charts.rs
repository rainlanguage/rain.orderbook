use crate::error::CommandResult;
use crate::commands::config::merge_configstrings;
use rain_orderbook_common::fuzz::*;
use rain_orderbook_app_settings::config::*;

#[tauri::command]
pub async fn make_charts(dotrain: String, settings: String) -> CommandResult<Vec<ChartData>> {
    let config = merge_configstrings(dotrain.clone(), settings)?;
    let final_config: Config = config.try_into()?;
    let mut fuzzer = FuzzRunner::new(dotrain.as_str(), final_config, None).await;

    let chart_data = fuzzer.build_chart_datas().await?;
    Ok(chart_data)
}