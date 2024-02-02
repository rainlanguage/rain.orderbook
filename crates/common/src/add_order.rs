use crate::transaction::{TransactionArgs, TransactionArgsError};
use alloy_ethers_typecast::transaction::{
    ReadableClientError, ReadableClientHttp, WritableClientError, WriteTransaction,
    WriteTransactionStatus,
};
use alloy_primitives::{hex::FromHexError, Address, U256};
use dotrain::{ComposeError, RainDocument, Store};
use rain_interpreter_dispair::{DISPair, DISPairError};
use rain_interpreter_parser::{Parser, ParserError, ParserV1};
use rain_meta::{
    ContentEncoding, ContentLanguage, ContentType, Error as RainMetaError, KnownMagic,
    RainMetaDocumentV1Item,
};
use rain_orderbook_bindings::IOrderBookV3::{addOrderCall, EvaluableConfigV3, OrderConfigV2, IO};
use serde_bytes::ByteBuf;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{scanner::ScanError, StrictYaml, StrictYamlLoader};
use thiserror::Error;

static REQUIRED_DOTRAIN_BODY_ENTRYPOINTS: [&str; 2] = ["calculate-io", "handle-io"];

#[derive(Error, Debug)]
pub enum AddOrderArgsError {
    #[error("frontmatter is not valid strict yaml: {0}")]
    FrontmatterInvalidYaml(#[from] ScanError),
    #[error("order frontmatter field is invalid: {0}")]
    FrontmatterFieldInvalid(String),
    #[error("order frontmatter field is missing: {0}")]
    FrontmatterFieldMissing(String),
    #[error(transparent)]
    DISPairError(#[from] DISPairError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[error(transparent)]
    RainMetaError(#[from] RainMetaError),
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
}

pub struct AddOrderArgs {
    /// Body of a Dotrain file describing an addOrder call
    /// File should have [strict yaml] frontmatter of the following structure
    ///
    /// ```yaml
    /// orderbook:
    ///     order:
    ///         deployer: 0x1111111111111111111111111111111111111111
    ///         validInputs:
    ///             - address: 0x2222222222222222222222222222222222222222
    ///               decimals: 18
    ///               vaultId: 0x1234
    ///         validOutputs:
    ///             - address: 0x5555555555555555555555555555555555555555
    ///               decimals: 8
    ///               vaultId: 0x5678
    /// ```
    pub dotrain: String,
}

impl AddOrderArgs {
    /// Parse an Io array from from frontmatter field (i.e. validInputs or validOutputs)
    fn try_parse_frontmatter_io(
        &self,
        io_yamls: StrictYaml,
        io_field_name: &str,
    ) -> Result<Vec<IO>, AddOrderArgsError> {
        io_yamls
            .into_vec()
            .ok_or(AddOrderArgsError::FrontmatterFieldMissing(format!(
                "orderbook.order.{}",
                io_field_name
            )))?
            .into_iter()
            .map(|io_yaml| -> Result<IO, AddOrderArgsError> {
                Ok(IO {
                    token: io_yaml["token"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterFieldMissing(format!(
                            "orderbook.order.{}.token",
                            io_field_name
                        )))?
                        .parse::<Address>()
                        .map_err(|_| {
                            AddOrderArgsError::FrontmatterFieldInvalid(format!(
                                "orderbook.order.{}.token",
                                io_field_name
                            ))
                        })?,
                    decimals: io_yaml["decimals"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterFieldMissing(format!(
                            "orderbook.order.{}.decimals",
                            io_field_name
                        )))?
                        .parse::<u8>()
                        .map_err(|_| {
                            AddOrderArgsError::FrontmatterFieldInvalid(format!(
                                "orderbook.order.{}.decimals",
                                io_field_name
                            ))
                        })?,
                    vaultId: io_yaml["vaultId"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterFieldMissing(format!(
                            "orderbook.order.{}.vault",
                            io_field_name
                        )))?
                        .parse::<U256>()
                        .map_err(|_| {
                            AddOrderArgsError::FrontmatterFieldInvalid(format!(
                                "orderbook.order.{}.vault",
                                io_field_name
                            ))
                        })?,
                })
            })
            .collect::<Result<Vec<IO>, AddOrderArgsError>>()
    }

    /// Parse dotrain frontmatter to extract deployer, validInputs and validOutputs
    fn try_parse_frontmatter(
        &self,
        frontmatter: &str,
    ) -> Result<(Address, Vec<IO>, Vec<IO>), AddOrderArgsError> {
        // Parse dotrain document frontmatter
        let frontmatter_yaml = StrictYamlLoader::load_from_str(frontmatter)
            .map_err(AddOrderArgsError::FrontmatterInvalidYaml)?;

        let deployer = frontmatter_yaml[0]["orderbook"]["order"]["deployer"]
            .as_str()
            .ok_or(AddOrderArgsError::FrontmatterFieldMissing(
                "orderbook.order.deployer".into(),
            ))?
            .parse::<Address>()
            .map_err(|_| {
                AddOrderArgsError::FrontmatterFieldInvalid("orderbook.order.deployer".into())
            })?;

        let valid_inputs: Vec<IO> = self.try_parse_frontmatter_io(
            frontmatter_yaml[0]["orderbook"]["order"]["validInputs"].clone(),
            "validInputs",
        )?;
        let valid_outputs: Vec<IO> = self.try_parse_frontmatter_io(
            frontmatter_yaml[0]["orderbook"]["order"]["validOutputs"].clone(),
            "validOutputs",
        )?;

        Ok((deployer, valid_inputs, valid_outputs))
    }

    /// Read parser address from deployer contract, then call parser to parse rainlang into bytecode and constants
    async fn try_parse_rainlang(
        &self,
        rpc_url: String,
        deployer: Address,
        rainlang: String,
    ) -> Result<(Vec<u8>, Vec<U256>), AddOrderArgsError> {
        let client = ReadableClientHttp::new_from_url(rpc_url)
            .map_err(AddOrderArgsError::ReadableClientError)?;
        let dispair = DISPair::from_deployer(deployer, client.clone())
            .await
            .map_err(AddOrderArgsError::DISPairError)?;

        let parser: ParserV1 = dispair.clone().into();
        let rainlang_parsed = parser
            .parse_text(rainlang.as_str(), client)
            .await
            .map_err(AddOrderArgsError::ParserError)?;

        Ok((rainlang_parsed.bytecode, rainlang_parsed.constants))
    }

    /// Generate RainlangSource meta
    fn try_generate_meta(&self, rainlang: String) -> Result<Vec<u8>, AddOrderArgsError> {
        let meta_doc = RainMetaDocumentV1Item {
            payload: ByteBuf::from(rainlang.as_bytes()),
            magic: KnownMagic::RainlangSourceV1,
            content_type: ContentType::OctetStream,
            content_encoding: ContentEncoding::None,
            content_language: ContentLanguage::None,
        };
        let meta_doc_bytes = RainMetaDocumentV1Item::cbor_encode_seq(
            &vec![meta_doc],
            KnownMagic::RainMetaDocumentV1,
        )
        .map_err(AddOrderArgsError::RainMetaError)?;

        Ok(meta_doc_bytes)
    }

    /// Generate an addOrder call from given dotrain
    async fn try_into_call(&self, rpc_url: String) -> Result<addOrderCall, AddOrderArgsError> {
        // Parse file into dotrain document
        let meta_store = Arc::new(RwLock::new(Store::default()));
        let raindoc = RainDocument::create(self.dotrain.clone(), Some(meta_store), None);
        let rainlang = raindoc.compose(&REQUIRED_DOTRAIN_BODY_ENTRYPOINTS)?;

        // Prepare call
        let (deployer, valid_inputs, valid_outputs) =
            self.try_parse_frontmatter(raindoc.front_matter().as_str())?;
        let (bytecode, constants) = self
            .try_parse_rainlang(rpc_url, deployer, rainlang.clone())
            .await?;
        let meta = self.try_generate_meta(rainlang)?;

        Ok(addOrderCall {
            config: OrderConfigV2 {
                validInputs: valid_inputs,
                validOutputs: valid_outputs,
                evaluableConfig: EvaluableConfigV3 {
                    deployer,
                    bytecode,
                    constants,
                },
                meta,
            },
        })
    }

    pub async fn execute<S: Fn(WriteTransactionStatus<addOrderCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), AddOrderArgsError> {
        let ledger_client = transaction_args.clone().try_into_ledger_client().await?;

        let add_order_call = self.try_into_call(transaction_args.clone().rpc_url).await?;
        let params = transaction_args
            .try_into_write_contract_parameters(add_order_call, transaction_args.orderbook_address)
            .await?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_frontmatter() {
        let frontmatter = "
orderbook:
    order:
        deployer: 0x1111111111111111111111111111111111111111
        validInputs:
            - token: 0x0000000000000000000000000000000000000001
              decimals: 18
              vaultId: 0x1
        validOutputs:
            - token: 0x0000000000000000000000000000000000000002
              decimals: 18
              vaultId: 0x2
";
        let args = AddOrderArgs { dotrain: "".into() };

        let (deployer, valid_inputs, valid_outputs) =
            args.try_parse_frontmatter(frontmatter).unwrap();

        assert_eq!(
            deployer,
            "0x1111111111111111111111111111111111111111"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            valid_inputs[0].token,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(valid_inputs[0].decimals, 18);
        assert_eq!(valid_inputs[0].vaultId, U256::from(1));
        assert_eq!(
            valid_outputs[0].token,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(valid_outputs[0].decimals, 18);
        assert_eq!(valid_outputs[0].vaultId, U256::from(2));
    }

    #[test]
    fn test_try_generate_meta() {
        let rainlang = String::from(
            "
#calculate-io
max-amount: 100e18,
price: 2e18;

#handle-io
max-amount: 100e18,
price: 2e18;
",
        );
        let args = AddOrderArgs { dotrain: "".into() };

        let meta_bytes = args.try_generate_meta(rainlang).unwrap();
        assert_eq!(
            meta_bytes,
            vec![
                255, 10, 137, 198, 116, 238, 120, 116, 163, 0, 88, 93, 10, 35, 99, 97, 108, 99,
                117, 108, 97, 116, 101, 45, 105, 111, 10, 109, 97, 120, 45, 97, 109, 111, 117, 110,
                116, 58, 32, 49, 48, 48, 101, 49, 56, 44, 10, 112, 114, 105, 99, 101, 58, 32, 50,
                101, 49, 56, 59, 10, 10, 35, 104, 97, 110, 100, 108, 101, 45, 105, 111, 10, 109,
                97, 120, 45, 97, 109, 111, 117, 110, 116, 58, 32, 49, 48, 48, 101, 49, 56, 44, 10,
                112, 114, 105, 99, 101, 58, 32, 50, 101, 49, 56, 59, 10, 1, 27, 255, 19, 16, 158,
                65, 51, 111, 242, 2, 120, 24, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110,
                47, 111, 99, 116, 101, 116, 45, 115, 116, 114, 101, 97, 109
            ]
        );
    }
}
