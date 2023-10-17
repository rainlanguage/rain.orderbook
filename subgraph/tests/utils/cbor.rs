use super::MagicNumber;
use anyhow::{anyhow, Result};
use ethers::types::{Bytes, U256};
use minicbor::data::Type;
use minicbor::decode::{Decode, Decoder, Error};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RainMapDoc {
    pub payload: Bytes,
    pub magic_number: U256,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub content_language: Option<String>,
}

impl RainMapDoc {
    fn bad_meta_map() -> Result<Self, Error> {
        return Err(Error::message("Bad rain meta map"));
    }
    fn no_meta_map() -> Result<Self, Error> {
        return Err(Error::message("It is not a rain meta map"));
    }
}

impl<'b> Decode<'b, ()> for RainMapDoc {
    fn decode(d: &mut Decoder<'b>, _: &mut ()) -> Result<Self, Error> {
        // Check what it's the current datatype.
        let datatype = d.datatype()?;

        if datatype == Type::Map {
            // Tecnically, it should not panic here since we already checked that
            // it is a map (the length map)
            let map_length = d.map()?.unwrap();

            if map_length < 2 || map_length > 5 {
                return Self::bad_meta_map();
            }

            let mut payload: Option<Bytes> = None;
            let mut magic_number: Option<U256> = None;
            let mut content_type: Option<String> = None;
            let mut content_encoding: Option<String> = None;
            let mut content_language: Option<String> = None;

            for _ in 0..map_length {
                let key = d.u8()?;

                match key {
                    0 => payload = Some(d.bytes()?.to_vec().into()),

                    1 => magic_number = Some(d.u64()?.into()),

                    2 => content_type = Some(d.str()?.to_string()),

                    3 => content_encoding = Some(d.str()?.to_string()),

                    4 => content_language = Some(d.str()?.to_string()),

                    // Does not allow other keys than the defnied by the metadata spec.
                    // See: https://github.com/rainprotocol/specs/blob/main/metadata-v1.md#header-name-aliases-cbor-map-keys
                    _ => return Self::bad_meta_map(),
                }
            }

            // This keys are mandatory
            if payload.is_none() || magic_number.is_none() {
                return Self::bad_meta_map();
            }

            Ok(RainMapDoc {
                payload: payload.unwrap(),
                magic_number: magic_number.unwrap(),
                content_type,
                content_encoding,
                content_language,
            })
        } else {
            // Since it's starting to decode and it's not a map, error.
            return Self::no_meta_map();
        }
    }
}

/// Receive a Rain Meta document with his prefix bytes and try to decode it.
pub fn decode_rain_meta(meta_data: Bytes) -> Result<Vec<RainMapDoc>> {
    let (doc_magic_number, cbor_data) = meta_data.split_at(8);

    if MagicNumber::rain_meta_document_v1() == doc_magic_number.to_vec() {
        let cbor_data = cbor_data.to_vec();

        return decode_cbor(cbor_data);
    }
    return Err(anyhow!("Cannot decode as a rain meta"));
}

pub fn decode_cbor(cbor_data: Vec<u8>) -> Result<Vec<RainMapDoc>> {
    let mut decoder = Decoder::new(&cbor_data);

    let mut all_docs: Vec<RainMapDoc> = vec![];

    while decoder.position() < decoder.input().len() {
        let doc: RainMapDoc = decoder.decode().unwrap();

        all_docs.push(doc);
    }

    return Ok(all_docs);
}
