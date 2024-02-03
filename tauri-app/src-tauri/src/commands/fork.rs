use rain_orderbook_common::fork::fork_parse_dotrain;

#[tauri::command]
pub async fn fork_parse(rainlang: &str, front_matter: &str, fork_url: &str, fork_block_number: u64) -> Result<(), String> {
    Ok(fork_parse_dotrain(rainlang, front_matter, fork_url, fork_block_number).await.map(|_| ())?)
}