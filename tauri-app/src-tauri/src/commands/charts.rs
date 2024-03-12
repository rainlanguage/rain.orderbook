use crate::error::CommandResult;
use rain_orderbook_common::fuzz::*;
use rain_orderbook_app_settings::string_structs::*;
use rain_orderbook_app_settings::config::*;
use alloy_primitives::U256;
use dotrain::{error::ComposeError, RainDocument, Rebind};
use serde_yaml::*;

#[tauri::command]
pub async fn make_charts(dotrain: &str, settings: Option<&str>) -> CommandResult<Vec<ChartData>> {
    let frontmatter = RainDocument::get_front_matter(dotrain).unwrap();
    let mut config = serde_yaml::from_str::<ConfigString>(frontmatter)?;
    let settings = settings.map(|s| serde_yaml::from_str::<ConfigString>(s));
    match settings {
        Some(Ok(s)) => config.merge(s)?,
        _ => (),
    };

    let final_config: Config = config.try_into()?;
    let mut fuzzer = FuzzRunner::new(dotrain, final_config, None).await;

    let chart_data = fuzzer.build_chart_datas().await?;
    Ok(chart_data)
}