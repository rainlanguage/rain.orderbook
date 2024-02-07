use super::add_order::REQUIRED_DOTRAIN_BODY_ENTRYPOINTS;
use super::fork::parse_dotrain_fork;
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
};
use dotrain_lsp::{
    lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    RainLanguageServices,
};
use once_cell::sync::Lazy;

/// static lang services instance
/// meta store instance can be taken from this for shared access to a unfied meta store across
/// all the dotrain usage in crate
pub static LANG_SERVICES: Lazy<RainLanguageServices> = Lazy::new(RainLanguageServices::default);

/// get hover for a given text document item
pub fn get_hover(text_document: &TextDocumentItem, position: Position) -> Option<Hover> {
    // @TODO - get rebinds
    // let rebinds = AddOrderArgs::try_parse_frontmatter(frontmatter)?.3;

    LANG_SERVICES.do_hover(text_document, position, None, None)
}

/// get completion items for a given text document item
pub fn get_completion(
    text_document: &TextDocumentItem,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    // @TODO - get rebinds
    // let rebinds = AddOrderArgs::try_parse_frontmatter(frontmatter)?.3;

    LANG_SERVICES.do_complete(text_document, position, None, None)
}

/// get problems for a given text document item
pub async fn get_problems(
    text_document: &TextDocumentItem,
    rpc_url: &str,
    block_number: u64,
) -> Vec<Problem> {
    // @TODO - get rebinds
    // let front_matter = match RainDocument::get_front_matter(&text_document.text) {
    //     Some(v) => v,
    //     None => {
    //         return vec![Problem {
    //             msg: "expected front matter".to_owned(),
    //             position: [0, 0],
    //             code: ErrorCode::NativeParserError,
    //         }]
    //     }
    // };
    // let rebinds = AddOrderArgs::try_parse_frontmatter(frontmatter)?.3;

    let rain_document = LANG_SERVICES.new_rain_document(text_document, None);
    let all_problems = rain_document.all_problems();
    if !all_problems.is_empty() {
        all_problems.iter().map(|&v| v.clone()).collect()
    } else {
        let front_matter = rain_document.front_matter();
        let rainlang = match rain_document.compose(REQUIRED_DOTRAIN_BODY_ENTRYPOINTS.as_slice()) {
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

        match parse_dotrain_fork(front_matter, &rainlang, rpc_url, block_number).await {
            Ok(_) => vec![],
            Err(e) => vec![Problem {
                msg: e.to_string(),
                position: [0, 0],
                code: ErrorCode::NativeParserError,
            }],
        }
    }
}
