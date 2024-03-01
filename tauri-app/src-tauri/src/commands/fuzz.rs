use crate::error::CommandResult;
use rain_orderbook_common::fuzz::*;
use rain_orderbook_app_settings::string_structs::*;
use rain_orderbook_app_settings::config::*;
use alloy_primitives::U256;
use dotrain::{error::ComposeError, RainDocument, Rebind};
use serde_yaml::*;

#[tauri::command]
pub async fn fuzz(scenario: &str, dotrain: &str, settings: Option<&str>) -> CommandResult<Vec<Vec<U256>>> {
    let frontmatter = RainDocument::get_front_matter(dotrain).unwrap();
    let settings = serde_yaml::from_str::<ConfigString>(frontmatter).unwrap();
    let config = settings
        .try_into()
        .map_err(|e| println!("{:?}", e))
        .unwrap();

    let mut fuzzer = FuzzRunner::new(dotrain, config, None).await;

    let result = fuzzer.run_scenario(scenario).await?;
    let stacks: Vec<Vec<U256>> = result.runs.iter().map(|run| run.stack.clone()).collect();

    Ok(stacks)
}