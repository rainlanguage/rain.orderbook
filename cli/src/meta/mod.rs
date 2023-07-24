pub mod solidity_abi;
pub mod op;
pub mod interpreter_caller;
pub mod rain;
pub mod normalize;
pub mod magic;
use magic::KnownMagic;
use serde::ser::{Serialize, Serializer, SerializeMap};

use strum::EnumIter;
use strum::EnumString;

#[derive(Copy, Clone, EnumString, EnumIter, strum::Display, Debug, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum KnownMeta {
    SolidityAbiV2,
    InterpreterCallerMetaV1,
    OpV1,
}

impl TryFrom<KnownMagic> for KnownMeta {
    type Error = anyhow::Error;
    fn try_from(magic: KnownMagic) -> anyhow::Result<Self> {
        match magic {
            KnownMagic::SolidityAbiV2 => Ok(KnownMeta::SolidityAbiV2),
            KnownMagic::InterpreterCallerMetaV1 => Ok(KnownMeta::InterpreterCallerMetaV1),
            KnownMagic::OpMetaV1 => Ok(KnownMeta::OpV1),
            _ => Err(anyhow::anyhow!("Unsupported magic {}", magic)),
        }
    }
}

#[derive(serde::Serialize, Copy, Clone, EnumString, EnumIter, strum::Display, Debug, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum ContentType {
    None,
    #[serde(rename = "application/json")]
    Json
}

#[derive(serde::Serialize, Copy, Clone, EnumString, EnumIter, strum::Display, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ContentEncoding {
    None,
    Identity,
    Deflate,
}

impl ContentEncoding {
    pub fn encode(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        Ok(match self {
            ContentEncoding::None | ContentEncoding::Identity => data,
            ContentEncoding::Deflate => deflate::deflate_bytes(&data),
        })
    }
}

#[derive(serde::Serialize, Copy, Clone, EnumString, EnumIter, strum::Display, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ContentLanguage {
    None,
    En,
}

#[derive(PartialEq, Debug)]
pub struct RainMetaDocumentV1Item {
    pub payload: serde_bytes::ByteBuf,
    pub magic: KnownMagic,
    pub content_type: ContentType,
    pub content_encoding: ContentEncoding,
    pub content_language: ContentLanguage,
}

impl RainMetaDocumentV1Item {
    fn len(&self) -> usize {
        let mut l = 2;
        if !matches!(self.content_type, ContentType::None) {
            l += 1;
        }
        if !matches!(self.content_encoding, ContentEncoding::None) {
            l += 1;
        }
        if !matches!(self.content_language, ContentLanguage::None) {
            l += 1;
        }
        l
    }
}

impl Serialize for RainMetaDocumentV1Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        map.serialize_entry(&0, &self.payload)?;
        map.serialize_entry(&1, &(self.magic as u64))?;
        match self.content_type {
            ContentType::None => { },
            content_type => map.serialize_entry(&2, &content_type)?,
        }
        match self.content_encoding {
            ContentEncoding::None => { },
            content_encoding => map.serialize_entry(&3, &content_encoding)?,
        }
        match self.content_language {
            ContentLanguage::None => { },
            content_language => map.serialize_entry(&4, &content_language)?,
        }
        map.end()
    }
}