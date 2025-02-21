use crate::commands::config::merge_configstrings;
use crate::error::CommandResult;
use rain_orderbook_app_settings::config::*;
use rain_orderbook_common::fuzz::*;

#[tauri::command]
pub async fn make_charts(dotrain: String, settings: String) -> CommandResult<ChartData> {
    let config = merge_configstrings(dotrain.clone(), settings).await?;
    let final_config: Config = config.try_into()?;
    let fuzzer = FuzzRunner::new(dotrain.as_str(), final_config.clone(), None).await;

    Ok(fuzzer.make_chart_data().await?)
}
