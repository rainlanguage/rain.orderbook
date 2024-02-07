use rain_orderbook_common::{
  dotrain::types::ast::Problem, 
  dotrain_lsp::lsp_types::{CompletionItem, Hover, Position, TextDocumentItem}, 
  lsp_services::{get_completion, get_hover, get_problems}
};

#[tauri::command]
pub fn provide_hover(text_document: TextDocumentItem, position: Position) -> Option<Hover> {
    get_hover(&text_document, position)
}

#[tauri::command]
pub fn provide_completion(text_document: TextDocumentItem, position: Position) -> Option<Vec<CompletionItem>> {
    get_completion(&text_document, position)
}

#[tauri::command]
pub async fn provide_problems(text_document: TextDocumentItem, rpc_url: &str, block_number: u64) -> Result<Vec<Problem>, ()> {
    Ok(get_problems(&text_document, rpc_url, block_number).await)
}