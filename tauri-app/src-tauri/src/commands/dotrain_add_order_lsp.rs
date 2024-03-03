use crate::error::CommandResult;
use alloy_primitives::Address;
use rain_orderbook_common::{
    dotrain::types::ast::Problem,
    dotrain_add_order_lsp::{completion, hover, problems},
    dotrain_lsp::lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
};
use std::collections::HashMap;

#[tauri::command]
pub fn call_lsp_hover(
    text_document: TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Hover> {
    hover(&text_document, position, bindings)
}

#[tauri::command]
pub fn call_lsp_completion(
    text_document: TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Vec<CompletionItem>> {
    completion(&text_document, position, bindings)
}

#[tauri::command]
pub async fn call_lsp_problems(
    text_document: TextDocumentItem,
    rpc_url: &str,
    block_number: Option<u64>,
    bindings: HashMap<String, String>,
    deployer: Option<Address>,
) -> CommandResult<Vec<Problem>> {
    Ok(problems(&text_document, rpc_url, block_number, bindings, deployer).await)
}
