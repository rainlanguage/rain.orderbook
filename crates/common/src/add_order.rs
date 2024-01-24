use alloy_primitives::Address;
use anyhow::Result;
use dotrain::{parser::RainDocument, types::Namespace};
use dotrain_interpreter_dispair::DISPair;
use dotrain_interpreter_parser::ParserV1;
use rain_interpreter_bindings::IParserV1::parseReturn;
use rain_orderbook_bindings::IOrderBookV3::{addOrderCall, EvaluableConfigV3, OrderConfigV2};
use std::{convert::TryInto, fs::read_to_string, path::PathBuf};
use strict_yaml_rust::StrictYamlLoader;

pub struct AddOrderArgs {
    /// Body of a Dotrain file describing an addOrder call
    /// File should have [strict yaml] frontmatter of the following structure
    /// 
    /// ```yaml
    /// orderbook:
    ///     order:
    ///         deployer: 0x11111111111111111111111111111111
    ///         validInputs:
    ///             - address: 0x22222222222222222222222222222222
    ///               decimals: 18
    ///               vaultId: 0x1234
    ///         validOutputs:
    ///             - address: 0x55555555555555555555555555555555
    ///               decimals: 8
    ///               vaultId: 0x5678
    /// ```
    pub dotrain: String,
}

impl AddOrderArgs {
    async fn try_into_call(self) -> Result<addOrderCall> {
        // Parse file into dotrain document
        let meta_store = Arc::new(RwLock::new(Store::default()));
        let raindoc = RainDocument::create(dotrain_body, meta_store);

        // Parse dotrain document frontmatter
        let frontmatter_yaml = StrictYamlLoader::load_from_str(raindoc.front_matter())?
        let deployer = &frontmatter_yaml[0]["orderbook"]["order"]["deployer"].parse::<Address>()?;
        let valid_inputs: Vec<IO> = &frontmatter_yaml[0]["orderbook"]["order"]["validInputs"].iter().map(|t| IO {
            token: t["address"].parse::<Address>()?,
            decimals: t["decimals"].parse::<u8>()?,
            vault_id: U256::from_str(t["vaultId"])?,
        }).collect();
        let valid_outputs: Vec<IO> = &frontmatter_yaml[0]["orderbook"]["order"]["validOutputs"].iter().map(|t| IO {
            token: t["address"].parse::<Address>()?,
            decimals: t["decimals"].parse::<u8>()?,
            vault_id: U256::from_str(t["vaultId"])?,
        }).collect();
        
        // Get DISPair addresses
        let dispair = DISPair::from_deployer(deployer).await?;

        // Parse rainlang text into bytecode + constants
        let parser: ParserV1 = dispair.into();
        let rainlang_parsed = parser.parse_text(raindoc.text()).await?;

        // @todo generate valid metadata including rainlangdoc.text
        // meta: arbitrary metadata https://github.com/rainlanguage/rain.metadata
        // use this library to convert rainlang source string to valid metadata
        // https://github.com/rainlanguage/rain.metadata/blob/main/crates/cli/src/meta/magic.rs
        // -- need to create a new magic code for rainlang source

        Ok(addOrderCall {
            config: OrderConfigV2 {
                validInputs: valid_inputs,
                validOutputs: valid_outputs,
                evaluableConfig: EvaluableConfigV3 {
                    deployer,
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
orderbook:
    order:
        deployer: 0x11111111111111111111111111111111
        validInputs:
            - token: 0x0000000000000000000000000000000000000001
            decimals: 16
            vaultId: 0x1
        validOutputs:
            - token: 0x0000000000000000000000000000000000000002
            decimals: 16
            vaultId: 0x2
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
