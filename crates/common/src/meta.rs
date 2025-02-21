use alloy::primitives::hex::{decode, FromHexError};
use rain_metadata::{Error as RainMetadataError, KnownMagic, RainMetaDocumentV1Item};
use rain_orderbook_subgraph_client::types::common::SgRainMetaV1;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TryDecodeRainlangSourceError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Meta bytes do not start with RainlangSourceV1 Magic")]
    MissingRainlangSourceV1,
    #[error(transparent)]
    RainMetadataError(#[from] RainMetadataError),
    #[error("Rainlang Source does not match rainlang")]
    RainlangSourceMismatch,
}

pub trait TryDecodeRainlangSource {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError>;
}

impl TryDecodeRainlangSource for SgRainMetaV1 {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError> {
        // Ensure meta has expected magic prefix
        let meta_bytes = decode(self.clone().0)?;
        if !meta_bytes
            .clone()
            .starts_with(&KnownMagic::RainMetaDocumentV1.to_prefix_bytes())
        {
            return Err(TryDecodeRainlangSourceError::MissingRainlangSourceV1);
        }

        // Decode meta to string
        let meta_bytes_slice = meta_bytes.as_slice();
        let rain_meta_document_item = RainMetaDocumentV1Item::cbor_decode(meta_bytes_slice)?;
        let rainlangsource_item = rain_meta_document_item
            .first()
            .ok_or(TryDecodeRainlangSourceError::MissingRainlangSourceV1)?;
        let rainlangsource = String::from_utf8(rainlangsource_item.payload.to_vec())?;

        Ok(rainlangsource)
    }
}
