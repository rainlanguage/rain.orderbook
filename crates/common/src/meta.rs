use alloy::primitives::hex::{decode, FromHexError};
use rain_metadata::{
    types::dotrain::instance_v1::DotrainInstanceV1, Error as RainMetadataError, KnownMagic,
    RainMetaDocumentV1Item,
};
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

#[derive(Error, Debug)]
pub enum TryDecodeDotrainInstanceError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Meta bytes do not start with RainMetaDocumentV1 Magic")]
    MissingRainMetaDocumentV1,
    #[error("DotrainInstanceV1 not found in metadata")]
    MissingDotrainInstanceV1,
    #[error(transparent)]
    RainMetadataError(#[from] RainMetadataError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

pub trait TryDecodeRainlangSource {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError>;
}

pub trait TryDecodeDotrainInstance {
    fn try_decode_dotrain_instance(
        &self,
    ) -> Result<DotrainInstanceV1, TryDecodeDotrainInstanceError>;
}

impl TryDecodeRainlangSource for String {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError> {
        // Ensure meta has expected magic prefix
        let meta_bytes = decode(self)?;
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

impl TryDecodeDotrainInstance for String {
    fn try_decode_dotrain_instance(
        &self,
    ) -> Result<DotrainInstanceV1, TryDecodeDotrainInstanceError> {
        // Ensure meta has expected magic prefix
        let meta_bytes = decode(self.clone())?;
        if !meta_bytes
            .clone()
            .starts_with(&KnownMagic::RainMetaDocumentV1.to_prefix_bytes())
        {
            return Err(TryDecodeDotrainInstanceError::MissingRainMetaDocumentV1);
        }

        // Decode meta document
        let meta_bytes_slice = meta_bytes.as_slice();
        let rain_meta_document_items = RainMetaDocumentV1Item::cbor_decode(meta_bytes_slice)?;

        // Find DotrainInstanceV1 item
        let dotrain_item = rain_meta_document_items
            .iter()
            .find(|item| item.magic == KnownMagic::DotrainInstanceV1)
            .ok_or(TryDecodeDotrainInstanceError::MissingDotrainInstanceV1)?;

        // Parse JSON payload to DotrainInstanceV1
        let dotrain_json = String::from_utf8(dotrain_item.payload.to_vec())?;
        let dotrain_instance: DotrainInstanceV1 = serde_json::from_str(&dotrain_json)?;

        Ok(dotrain_instance)
    }
}

#[cfg(test)]
mod tests {
    use rain_orderbook_subgraph_client::types::common::SgBytes;
    use rain_orderbook_subgraph_client::types::common::SgRainMetaV1;

    use super::*;

    const RAINLANG_SOURCE: &str = r#"/* 0. calculate-io */ 
using-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
max-output: max-value(),
io: if(
  equal-to(
    output-token()
    0x1d80c49bbbcd1c0911346656b529df9e5c2f783d
  )
  12
  inv(12)
);

/* 1. handle-io */ 
:;"#;
    const META: &str = "0xff0a89c674ee7874a30058ef2f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a6d61782d6f75747075743a206d61782d76616c756528292c0a696f3a206966280a2020657175616c2d746f280a202020206f75747075742d746f6b656e28290a202020203078316438306334396262626364316330393131333436363536623532396466396535633266373833640a2020290a202031320a2020696e76283132290a293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d";

    #[test]
    fn test_try_decode_rainlangsource() {
        let meta: SgRainMetaV1 = SgBytes(META.to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap();
        assert_eq!(source, RAINLANG_SOURCE);
    }

    #[test]
    fn test_try_decode_rainlangsource_missing() {
        let meta: SgRainMetaV1 = SgBytes("".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::MissingRainlangSourceV1.to_string()
        );
    }

    #[test]
    fn test_try_decode_rainlangsource_invalid() {
        let meta: SgRainMetaV1 = SgBytes("invalid".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::FromHexError(FromHexError::OddLength).to_string()
        );
    }

    #[test]
    fn test_try_decode_rainlangsource_corrupt() {
        let meta: SgRainMetaV1 = SgBytes("0xff0a89c674ee7874".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::RainMetadataError(RainMetadataError::CorruptMeta)
                .to_string()
        );
    }

    #[test]
    fn test_try_decode_dotrain_instance() {
        use alloy::primitives::B256;
        use rain_metadata::{
            ContentEncoding, ContentLanguage, ContentType, RainMetaDocumentV1Item,
        };
        use serde_bytes::ByteBuf;
        use std::collections::BTreeMap;

        // Create test DotrainInstanceV1
        let dotrain_instance_data = DotrainInstanceV1 {
            dotrain_hash: B256::from_slice(&[42u8; 32]),
            field_values: BTreeMap::from([(
                "amount".to_string(),
                rain_metadata::types::dotrain::instance_v1::ValueCfg {
                    id: "amount_field".to_string(),
                    name: Some("Amount".to_string()),
                    value: "100".to_string(),
                },
            )]),
            deposits: BTreeMap::new(),
            select_tokens: BTreeMap::new(),
            vault_ids: BTreeMap::new(),
            selected_deployment: "test_deployment".to_string(),
        };

        // Create metadata with both RainlangSource and DotrainInstanceV1
        let rainlang_meta_doc = RainMetaDocumentV1Item {
            payload: ByteBuf::from(RAINLANG_SOURCE.as_bytes()),
            magic: KnownMagic::RainlangSourceV1,
            content_type: ContentType::OctetStream,
            content_encoding: ContentEncoding::None,
            content_language: ContentLanguage::None,
        };

        let dotrain_meta_doc: RainMetaDocumentV1Item = dotrain_instance_data.clone().into();

        let meta_docs = vec![rainlang_meta_doc, dotrain_meta_doc];
        let meta_bytes =
            RainMetaDocumentV1Item::cbor_encode_seq(&meta_docs, KnownMagic::RainMetaDocumentV1)
                .unwrap();
        let meta_hex = alloy::primitives::hex::encode(&meta_bytes);

        // Test parsing DotrainInstanceV1
        let meta: SgRainMetaV1 = SgBytes(meta_hex);
        let parsed_instance = meta.0.try_decode_dotrain_instance().unwrap();

        // Verify the parsed data
        assert_eq!(
            parsed_instance.dotrain_hash,
            dotrain_instance_data.dotrain_hash
        );
        assert_eq!(parsed_instance.field_values.len(), 1);
        assert_eq!(parsed_instance.selected_deployment, "test_deployment");
        assert_eq!(parsed_instance.field_values["amount"].id, "amount_field");
        assert_eq!(parsed_instance.field_values["amount"].value, "100");
    }

    #[test]
    fn test_try_decode_dotrain_instance_missing() {
        let meta: SgRainMetaV1 = SgBytes("".to_string());
        let instance = meta.0.try_decode_dotrain_instance().unwrap_err();
        assert_eq!(
            instance.to_string(),
            TryDecodeDotrainInstanceError::MissingRainMetaDocumentV1.to_string()
        );
    }

    #[test]
    fn test_try_decode_dotrain_instance_invalid() {
        let meta: SgRainMetaV1 = SgBytes("invalid".to_string());
        let instance = meta.0.try_decode_dotrain_instance().unwrap_err();
        assert_eq!(
            instance.to_string(),
            TryDecodeDotrainInstanceError::FromHexError(FromHexError::OddLength).to_string()
        );
    }

    #[test]
    fn test_try_decode_dotrain_instance_not_found() {
        // Use metadata that only contains RainlangSource, not DotrainInstanceV1
        let meta: SgRainMetaV1 = SgBytes(META.to_string());
        let instance = meta.0.try_decode_dotrain_instance().unwrap_err();
        assert_eq!(
            instance.to_string(),
            TryDecodeDotrainInstanceError::MissingDotrainInstanceV1.to_string()
        );
    }

    #[test]
    fn test_try_decode_dotrain_instance_corrupt() {
        let meta: SgRainMetaV1 = SgBytes("0xff0a89c674ee7874".to_string());
        let instance = meta.0.try_decode_dotrain_instance().unwrap_err();
        assert_eq!(
            instance.to_string(),
            TryDecodeDotrainInstanceError::RainMetadataError(RainMetadataError::CorruptMeta)
                .to_string()
        );
    }
}
