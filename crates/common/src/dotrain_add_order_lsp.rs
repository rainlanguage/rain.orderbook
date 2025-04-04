#[cfg(not(target_family = "wasm"))]
use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
#[cfg(not(target_family = "wasm"))]
use crate::rainlang::parse_rainlang_on_fork;
#[cfg(not(target_family = "wasm"))]
use alloy::primitives::Address;
use dotrain::Rebind;
#[cfg(not(target_family = "wasm"))]
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
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

pub struct DotrainAddOrderLsp {
    text_document: TextDocumentItem,
    rebinds: Option<Vec<Rebind>>,
}

impl DotrainAddOrderLsp {
    pub fn new(text_document: TextDocumentItem, bindings: HashMap<String, String>) -> Self {
        let rebinds = if !bindings.is_empty() {
            Some(
                bindings
                    .iter()
                    .map(|(key, value)| Rebind(key.clone(), value.clone()))
                    .collect(),
            )
        } else {
            None
        };

        Self {
            text_document: text_document.clone(),
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
    #[cfg(not(target_family = "wasm"))]
    pub async fn problems(
        &self,
        rpc_url: &str,
        block_number: Option<u64>,
        deployer: Option<Address>,
    ) -> Vec<Problem> {
        let rain_document =
            LANG_SERVICES.new_rain_document(&self.text_document, self.rebinds.clone());
        let mut bindings_problems = rain_document
            .bindings_problems()
            .iter()
            .map(|&v| v.clone())
            .collect::<Vec<_>>();
        let top_problems = rain_document.problems();
        if !top_problems.is_empty() {
            bindings_problems.extend(top_problems.to_vec());
            bindings_problems
        } else {
            let rainlang = match rain_document.compose(&ORDERBOOK_ORDER_ENTRYPOINTS) {
                Ok(v) => v,
                Err(e) => match e {
                    ComposeError::Reject(msg) => {
                        bindings_problems.push(Problem {
                            msg,
                            position: [0, 0],
                            code: ErrorCode::NativeParserError,
                        });
                        return bindings_problems;
                    }
                    ComposeError::Problems(problems) => {
                        for p in problems {
                            if bindings_problems.iter().all(|v| p != *v) {
                                bindings_problems.push(p)
                            }
                        }
                        return bindings_problems;
                    }
                },
            };

            if let Some(deployer_address) = deployer {
                if let Err(e) =
                    parse_rainlang_on_fork(&rainlang, rpc_url, block_number, deployer_address).await
                {
                    bindings_problems.push(Problem {
                        msg: e.to_string(),
                        position: [0, 0],
                        code: ErrorCode::NativeParserError,
                    })
                };
            } else {
                bindings_problems.push(Problem {
                    msg: "Choose a deployment to get Rainlang diagnostics".to_owned(),
                    position: [0, 0],
                    code: ErrorCode::NativeParserError,
                });
            }
            bindings_problems
        }
    }
}
