use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use crate::frontmatter::try_parse_frontmatter_rebinds;
use crate::rainlang::parse_rainlang_on_fork;
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
    RainDocument, Rebind,
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

pub struct DotrainAddOrderLsp {
    text_document: TextDocumentItem,
    frontmatter: String,
    rebinds: Option<Vec<Rebind>>,
}

impl DotrainAddOrderLsp {
    pub fn new(text_document: TextDocumentItem) -> Self {
        let frontmatter = RainDocument::get_front_matter(&text_document.text);
        let rebinds = frontmatter.and_then(try_parse_frontmatter_rebinds);

        Self {
            text_document: text_document.clone(),
            frontmatter: frontmatter.unwrap_or("").to_string(),
            rebinds,
        }
    }

    /// get hover for a given text document item
    pub fn hover(&self, position: Position) -> Option<Hover> {
        LANG_SERVICES.do_hover(&self.text_document, position, None, self.rebinds.clone())
    }

    /// get completion items for a given text document item
    pub fn completion(&self, position: Position) -> Option<Vec<CompletionItem>> {
        LANG_SERVICES.do_complete(&self.text_document, position, None, self.rebinds.clone())
    }

    /// get problems for a given text document item
    pub async fn problems(&self, rpc_url: &str, block_number: Option<u64>) -> Vec<Problem> {
        let rain_document =
            LANG_SERVICES.new_rain_document(&self.text_document, self.rebinds.clone());
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

            parse_rainlang_on_fork(&self.frontmatter, &rainlang, rpc_url, block_number)
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
}
