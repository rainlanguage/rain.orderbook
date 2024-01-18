use alloy_primitives::Address;
use anyhow::Result;
use dotrain::{parser::RainLangDocument, types::Namespace};
use dotrain_interpreter_dispair::DISPair;
use dotrain_interpreter_parser::ParserV1;
use rain_interpreter_bindings::IParserV1::parseReturn;
use rain_orderbook_bindings::IOrderBookV3::{addOrderCall, EvaluableConfigV3, OrderConfigV2};
use std::{convert::TryInto, fs::read_to_string, path::PathBuf};

pub struct AddOrderArgs {
    /// Path to a .rain file
    pub dotrain_path: PathBuf,
}

impl AddOrderArgs {
    async fn try_into_call(self) -> Result<addOrderCall> {
        let rainlangdoc = RainLangDocument::create(dotrain_body, Namespace::Dispair, None);

        // @todo use dotrain parser to extract frontmatter
        //  - parse frontmatter as yaml
        //  - read deployer from yaml
        //  - read validInputs from yaml
        //  - read validOutputs from yaml

        // Get DISPair addresses
        let deployer_address = self.deployer.parse::<Address>()?;
        let dispair = DISPair::from_deployer(deployer_address).await?;

        // Parse rainlang into bytecode + constants
        let parser: ParserV1 = dispair.into();
        let rainlang_parsed = parser.parse_text(rainlangdoc.text).await?;

        // @todo generate valid metadata including rainlangdoc.text
        // meta: arbitrary metadata https://github.com/rainlanguage/rain.metadata
        // use this library to convert rainlang source string to valid metadata
        // https://github.com/rainlanguage/rain.metadata/blob/main/crates/cli/src/meta/magic.rs
        // -- need to create a new magic code for rainlang source

        Ok(addOrderCall {
            config: OrderConfigV2 {
                validInputs: vec![],
                validOutputs: vec![],
                evaluableConfig: EvaluableConfigV3 {
                    deployer: deployer_address,
                    bytecode: rainlang_parsed.bytecode,
                    constants: rainlang_parsed.constants,
                },
                meta: vec![],
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use alloy_primitives::{hex, Address, U256};
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_order_args_try_into() {
        let dotrain_text = "
---
- validInputs:
    - token: 0x0000000000000000000000000000000000000001
      decimals: 16
      vaultId: 1
- validOutputs:
    - token: 0x0000000000000000000000000000000000000002
      decimals: 16
      vaultId: 2
---
start-time: 160000,
end-time: 160600,
start-price: 100e18,
rate: 1e16

:ensure(
    every(
        gt(now() start-time))
        lt(now() end-time)),
    )
),

elapsed: sub(now() start-time),

max-amount: 1000e18,
price: sub(start-price mul(rate elapsed))
";
        let mut dotrain_file = NamedTempFile::new().unwrap();
        dotrain_file.reopen().unwrap();
        dotrain_file.write_all(dotrain_text.as_bytes()).unwrap();

        let args = AddOrderArgs {
            dotrain_path: PathBuf::from(dotrain_file.path()),
            deployer: Address::repeat_byte(0x11).to_string(),
        };

        let result: Result<addOrderCall, _> = args.try_into();

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let add_order_call = result.unwrap();
        assert_eq!(
            add_order_call.config.validInputs[0].token,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(add_order_call.config.validInputs[0].decimals, 16);
        assert_eq!(add_order_call.config.validInputs[0].vaultId, U256::from(1));

        assert_eq!(
            add_order_call.config.validOutputs[0].token,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(add_order_call.config.validOutputs[0].decimals, 16);
        assert_eq!(add_order_call.config.validOutputs[0].vaultId, U256::from(2));

        assert_eq!(
            add_order_call.config.evaluableConfig.deployer,
            hex!("1111111111111111111111111111111111111111")
        );
        // @todo test against properly encoded dotrain bytecode
        assert_eq!(
            add_order_call.config.evaluableConfig.bytecode,
            vec![0u8; 32]
        );

        // @todo test against properly encoded dotrain constants
        assert_eq!(
            add_order_call.config.evaluableConfig.constants,
            vec![
                U256::from(160000),
                U256::from(160600),
                U256::from(100e18),
                U256::from(1e16),
            ]
        );

        // @todo add example meta to rainlang and test it is parsed properly
    }
}
