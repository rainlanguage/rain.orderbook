use crate::transaction::{TransactionArgs, TransactionArgsError};
use alloy_ethers_typecast::transaction::{
    ReadableClientError, ReadableClientHttp, WritableClientError, WriteTransaction,
    WriteTransactionStatus,
};
use alloy_primitives::{Address, U256};
use dotrain::{RainDocument, Store};
use rain_interpreter_dispair::{DISPair, DISPairError};
use rain_interpreter_parser::{Parser, ParserError, ParserV1};
use rain_orderbook_bindings::IOrderBookV3::{addOrderCall, EvaluableConfigV3, OrderConfigV2, IO};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{scanner::ScanError, StrictYaml, StrictYamlLoader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddOrderArgsError {
    #[error("frontmatter is not valid strict yaml: {0}")]
    FrontmatterInvalidYaml(#[from] ScanError),
    #[error("deployer field is not a valid Address")]
    FrontmatterDeployerInvalid,
    #[error("validInputs field is not a valid array")]
    FrontmatterInputsInvalid,
    #[error("validOutputs field is not a valid array")]
    FrontmatterOutputsInvalid,
    #[error("validInputs or validOutputs is not a valid dict")]
    FrontmatterIOInvalid,
    #[error("token field is missing")]
    FrontmatterIOTokenMissing,
    #[error("token field is invalid")]
    FrontmatterIOTokenInvalid,
    #[error("decimals field is missing")]
    FrontmatterIODecimalsMissing,
    #[error("decimals field is not a valid u8")]
    FrontmatterIODecimalsInvalid,
    #[error("vaultId field is missing")]
    FrontmatterIOVaultIdMissing,
    #[error("vaultId field is not a valid U256")]
    FrontmatterIOVaultIdInvalid,
    #[error(transparent)]
    DISPairError(#[from] DISPairError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    FromHexError(#[from] alloy_primitives::hex::FromHexError),
    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),
    #[error("TransactionArgs error: {0}")]
    TransactionArgs(#[from] TransactionArgsError),
}

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
    fn parse_io(&self, io_yamls: StrictYaml) -> Result<Vec<IO>, AddOrderArgsError> {
        io_yamls
            .into_vec()
            .ok_or(AddOrderArgsError::FrontmatterInputsInvalid)?
            .into_iter()
            .map(|io_yaml| -> Result<IO, AddOrderArgsError> {
                Ok(IO {
                    token: io_yaml["token"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterIOTokenMissing)?
                        .parse::<Address>()
                        .map_err(|_| AddOrderArgsError::FrontmatterIOTokenInvalid)?,
                    decimals: io_yaml["decimals"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterIODecimalsMissing)?
                        .parse::<u8>()
                        .map_err(|_| AddOrderArgsError::FrontmatterIODecimalsInvalid)?,
                    vaultId: io_yaml["vaultId"]
                        .as_str()
                        .ok_or(AddOrderArgsError::FrontmatterIOVaultIdMissing)?
                        .parse::<U256>()
                        .map_err(|_| AddOrderArgsError::FrontmatterIOVaultIdInvalid)?,
                })
            })
            .collect::<Result<Vec<IO>, AddOrderArgsError>>()
    }

    async fn try_into_call(&self, rpc_url: String) -> Result<addOrderCall, AddOrderArgsError> {
        // Parse file into dotrain document
        let meta_store = Arc::new(RwLock::new(Store::default()));
        let raindoc = RainDocument::create(self.dotrain.clone(), Some(meta_store));

        // Parse dotrain document frontmatter
        let frontmatter_yaml = StrictYamlLoader::load_from_str(raindoc.front_matter())
            .map_err(AddOrderArgsError::FrontmatterInvalidYaml)?;

        let deployer = frontmatter_yaml[0]["orderbook"]["order"]["deployer"]
            .as_str()
            .ok_or(AddOrderArgsError::FrontmatterDeployerInvalid)?
            .parse::<Address>()
            .map_err(|_| AddOrderArgsError::FrontmatterDeployerInvalid)?;

        let valid_inputs: Vec<IO> =
            self.parse_io(frontmatter_yaml[0]["orderbook"]["order"]["validInputs"].clone())?;
        let valid_outputs: Vec<IO> =
            self.parse_io(frontmatter_yaml[0]["orderbook"]["order"]["validOutputs"].clone())?;

        // Get DISPair addresses
        let client = ReadableClientHttp::new_from_url(rpc_url)
            .map_err(AddOrderArgsError::ReadableClientError)?;

        // Read parser address from dispair contract
        let dispair = DISPair::from_deployer(deployer, client.clone())
            .await
            .map_err(AddOrderArgsError::DISPairError)?;

        // Parse rainlang text into bytecode + constants
        let parser: ParserV1 = dispair.into();
        let rainlang_parsed = parser
            .parse_text(raindoc.body(), client)
            .await
            .map_err(AddOrderArgsError::ParserError)?;

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

    pub async fn execute<S: Fn(WriteTransactionStatus<addOrderCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), AddOrderArgsError> {
        let ledger_client = transaction_args
            .clone()
            .try_into_ledger_client()
            .await
            .map_err(AddOrderArgsError::TransactionArgs)?;

        let add_order_call = self
            .try_into_call(transaction_args.clone().rpc_url)
            .await
            .unwrap();
        let params = transaction_args
            .try_into_write_contract_parameters(add_order_call, transaction_args.orderbook_address)
            .await
            .map_err(AddOrderArgsError::TransactionArgs)?;

        WriteTransaction::new(ledger_client.client, params, 4, transaction_status_changed)
            .execute()
            .await
            .map_err(AddOrderArgsError::WritableClientError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, Address, U256};

    #[tokio::test]
    async fn test_add_order_args_try_into() {
        let dotrain = String::from(
            "
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
",
        );
        let args = AddOrderArgs { dotrain };
        let add_order_call: addOrderCall = args.try_into_call(String::from("myrpc")).await.unwrap();

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
