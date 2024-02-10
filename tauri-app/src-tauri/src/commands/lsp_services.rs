use rain_orderbook_common::{
    dotrain::types::ast::Problem,
    dotrain_lsp::lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    lsp_services::{DotrainAddOrderLsp},
};
use crate::error::CommandResult;

#[tauri::command]
pub fn provide_hover(text_document: TextDocumentItem, position: Position) -> Option<Hover> {
    DotrainAddOrderLsp::new(text_document).hover(position)
}

#[tauri::command]
pub fn provide_completion(
    text_document: TextDocumentItem,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    DotrainAddOrderLsp::new(text_document).completion(position)
}

#[tauri::command]
pub async fn provide_problems(
    text_document: TextDocumentItem,
    rpc_url: &str,
    block_number: Option<u64>,
) -> CommandResult<Vec<Problem>> {
    Ok(DotrainAddOrderLsp::new(text_document).problems(rpc_url, block_number).await)
}
