use crate::error::CommandResult;
use alloy::primitives::Address;
use rain_orderbook_common::{
    dotrain::types::ast::Problem,
    dotrain_add_order_lsp::DotrainAddOrderLsp,
    dotrain_lsp::lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
};
use std::collections::HashMap;

#[tauri::command]
pub fn call_lsp_hover(
    text_document: TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Hover> {
    DotrainAddOrderLsp::new(text_document, bindings).hover(position)
}

#[tauri::command]
pub fn call_lsp_completion(
    text_document: TextDocumentItem,
    position: Position,
    bindings: HashMap<String, String>,
) -> Option<Vec<CompletionItem>> {
    DotrainAddOrderLsp::new(text_document, bindings).completion(position)
}

#[tauri::command]
pub async fn call_lsp_problems(
    text_document: TextDocumentItem,
    rpc_url: &str,
    block_number: Option<u64>,
    bindings: HashMap<String, String>,
    deployer: Option<Address>,
) -> CommandResult<Vec<Problem>> {
    Ok(DotrainAddOrderLsp::new(text_document, bindings)
        .problems(rpc_url, block_number, deployer)
        .await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_test_fixtures::LocalEvm;
    use url::Url;

    fn get_text_document(text: &str) -> TextDocumentItem {
        TextDocumentItem {
            uri: Url::parse("file:///temp.rain").unwrap(),
            language_id: "rainlang".to_string(),
            version: 0,
            text: text.to_string(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_call_lsp_problems_no_problems() {
        let local_evm = LocalEvm::new().await;

        let rainlang = r#"
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#;
        let problems = call_lsp_problems(
            get_text_document(rainlang),
            &local_evm.url(),
            None,
            HashMap::new(),
            Some(*local_evm.deployer.address()),
        )
        .await
        .unwrap();
        assert_eq!(problems.len(), 0);
    }

    #[tokio::test]
    async fn test_call_lsp_problems() {
        let local_evm = LocalEvm::new().await;

        let dotrain = r#"
        raindex-version: 0
        
        tokens:
          token1:
            network: flare
            address: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
            decimals: 18
          token2:
            network: flare
            address: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
            decimals: 18
        
        orders:
          flare1:
            orderbook: flare
            inputs:
              - token: token1
            outputs:
              - token: token2
          flareaaaaa:
            orderbook: flare
            inputs:
              - token: token1
            outputs:
              - token: token1
          flare2:
            orderbook: flare
            inputs:
              - token: token2
            outputs:
              - token: token1
        
        scenarios:
          flare:
            orderbook: flare
            runs: 1
            bindings:
              raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
              fixed-io-output-token: ${order.outputs.0.token.address}
        
        deployments:
          flare1:
            order: flare1
            scenario: flare
            
        ---
        "test"
        
        #raindex-subparser !The subparser to use.
        
        #fixed-io !The io ratio for the limit order.
        #fixed-io-output-token !The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.
        
        #calculate-io
        
        123;
        invalid-text;
        
        using-words-from raindex-subparser
        
        _: ensure(ABC),
        
        max-output: max-value(),
        io: if(
          equal-to(
            output-token()
            fixed-io-output-token
          )
          fixed-io
          inv(test)
        );
        
        #handle-io
        :;
        
        #handle-add-order
        :;
        "#;

        let problems = call_lsp_problems(
            get_text_document(dotrain),
            &local_evm.url(),
            None,
            HashMap::from([
                (
                    "raindex-subparser".to_string(),
                    "0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC".to_string(),
                ),
                (
                    "fixed-io-output-token".to_string(),
                    "0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d".to_string(),
                ),
            ]),
            Some(*local_evm.deployer.address()),
        )
        .await
        .unwrap();

        let expected_msgs = [
            "invalid expression line",
            "invalid expression line",
            "invalid word pattern: ABC",
            "elided binding 'fixed-io': The io ratio for the limit order.",
            "undefined word: test",
            "unexpected token",
        ];
        let actual_msgs: Vec<String> = problems.iter().map(|p| p.msg.clone()).collect();

        assert_eq!(problems.len(), 6);

        for (actual, expected) in actual_msgs.iter().zip(expected_msgs.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
