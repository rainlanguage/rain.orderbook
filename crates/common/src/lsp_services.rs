use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use crate::frontmatter::try_parse_frontmatter_rebinds;
use crate::rainlang::parse_rainlang_on_fork;
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
    RainDocument,
};
use dotrain_lsp::{
    lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    RainLanguageServices,
};
use once_cell::sync::Lazy;

/// static lang services instance
/// meta store instance can be taken from this for shared access to a unfied meta store across
/// all the dotrain usage in this crate
pub static LANG_SERVICES: Lazy<RainLanguageServices> = Lazy::new(RainLanguageServices::default);

/// get hover for a given text document item
pub fn get_hover(text_document: &TextDocumentItem, position: Position) -> Option<Hover> {
    let rebinds =
        RainDocument::get_front_matter(&text_document.text).and_then(try_parse_frontmatter_rebinds);

    LANG_SERVICES.do_hover(text_document, position, None, rebinds)
}

/// get completion items for a given text document item
pub fn get_completion(
    text_document: &TextDocumentItem,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    let rebinds =
        RainDocument::get_front_matter(&text_document.text).and_then(try_parse_frontmatter_rebinds);

    LANG_SERVICES.do_complete(text_document, position, None, rebinds)
}

/// get problems for a given text document item
pub async fn get_problems(
    text_document: &TextDocumentItem,
    rpc_url: &str,
    block_number: u64,
) -> Vec<Problem> {
    let frontmatter = RainDocument::get_front_matter(&text_document.text).unwrap_or("");
    let rebinds = try_parse_frontmatter_rebinds(frontmatter);

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

        parse_rainlang_on_fork(frontmatter, &rainlang, rpc_url, Some(block_number))
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
    }
}
