use alloy::primitives::hex::{decode, FromHexError};
use rain_metadata::{Error as RainMetadataError, KnownMagic, RainMetaDocumentV1Item};
use rain_orderbook_subgraph_client::types::common::RainMetaV1;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TryDecodeOrderMetaError {
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

pub trait TryDecodeOrderMeta {
    fn try_decode_meta(&self) -> Result<(String, Option<String>), TryDecodeOrderMetaError>;
}

impl TryDecodeOrderMeta for RainMetaV1 {
    fn try_decode_meta(&self) -> Result<(String, Option<String>), TryDecodeOrderMetaError> {
        // Ensure meta has expected magic prefix
        let meta_bytes = decode(self.clone().0)?;
        if !meta_bytes
            .clone()
            .starts_with(&KnownMagic::RainMetaDocumentV1.to_prefix_bytes())
        {
            return Err(TryDecodeOrderMetaError::MissingRainlangSourceV1);
        }

        // Decode meta to rainlang and dotrain
        let rain_meta_document_item = RainMetaDocumentV1Item::cbor_decode(&meta_bytes)?;
        let rainlangsource_item = rain_meta_document_item
            .first()
            .ok_or(TryDecodeOrderMetaError::MissingRainlangSourceV1)?;
        if rainlangsource_item.magic != KnownMagic::RainlangSourceV1
            && rainlangsource_item.magic != KnownMagic::RainlangV1
        {
            return Err(TryDecodeOrderMetaError::MissingRainlangSourceV1);
        }
        // decompresses and unpacks into the original string
        let rainlangsource = rainlangsource_item.clone().unpack_into::<String>()?;

        let mut dotrain = None;
        for item in &rain_meta_document_item[1..] {
            if item.magic == KnownMagic::DotrainV1 {
                // decompresses and unpacks into the original string
                dotrain = Some(item.clone().unpack_into::<String>()?);
                break;
            }
        }

        Ok((rainlangsource, dotrain))
    }
}
