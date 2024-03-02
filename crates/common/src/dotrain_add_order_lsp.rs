use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use crate::rainlang::parse_rainlang_on_fork;
use alloy_primitives::Address;
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
    Rebind,
};
use dotrain_lsp::{
    lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    RainLanguageServices,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// static lang services instance
/// meta store instance can be taken from this for shared access to a unfied meta store across
/// all the dotrain usage in this crate
pub static LANG_SERVICES: Lazy<RainLanguageServices> = Lazy::new(RainLanguageServices::default);

/// get hover for a given text document item
pub fn hover(
    text_document: &TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Hover> {
    let mut rebinds = None;
    if !bindings.is_empty() {
        rebinds = Some(
            bindings
                .iter()
                .map(|(key, value)| Rebind(key.clone(), value.clone()))
                .collect(),
        );
    };
    LANG_SERVICES.do_hover(text_document, position, None, rebinds)
}

/// get completion items for a given text document item
pub fn completion(
    text_document: &TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Vec<CompletionItem>> {
    let mut rebinds = None;
    if !bindings.is_empty() {
        rebinds = Some(
            bindings
                .iter()
                .map(|(key, value)| Rebind(key.clone(), value.clone()))
                .collect(),
        );
    };
    LANG_SERVICES.do_complete(text_document, position, None, rebinds)
}

/// get problems for a given text document item
pub async fn problems(
    text_document: &TextDocumentItem,
    rpc_url: &str,
    block_number: Option<u64>,
    bindings: HashMap<String, String>,
    deployer: Option<Address>,
) -> Vec<Problem> {
    let mut rebinds = None;
    if !bindings.is_empty() {
        rebinds = Some(
            bindings
                .iter()
                .map(|(key, value)| Rebind(key.clone(), value.clone()))
                .collect(),
        );
    };
    let rain_document = LANG_SERVICES.new_rain_document(text_document, rebinds);
    let all_problems = rain_document.all_problems();
    if !all_problems.is_empty() {
        all_problems.iter().map(|&v| v.clone()).collect()
    } else {
        let rainlang = match rain_document.compose(&ORDERBOOK_ORDER_ENTRYPOINTS) {
            Ok(v) => v,
            Err(e) => match e {
                ComposeError::Reject(msg) => {
                    return vec![Problem {
                        msg,
                        position: [0, 0],
                        code: ErrorCode::NativeParserError,
                    }]
                }
                ComposeError::Problems(problems) => return problems,
            },
        };

        if let Some(deployer_add) = deployer {
            parse_rainlang_on_fork(&rainlang, rpc_url, block_number, deployer_add)
                .await
                .map_or_else(
                    |e| {
                        vec![Problem {
                            msg: e.to_string(),
                            position: [0, 0],
                            code: ErrorCode::NativeParserError,
                        }]
                    },
                    |_| vec![],
                )
        } else {
            vec![Problem {
                msg: "undefined deployer address".to_owned(),
                position: [0, 0],
                code: ErrorCode::NativeParserError,
            }]
        }
    }
}
