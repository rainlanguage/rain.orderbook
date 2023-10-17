use super::MagicNumber;
use anyhow::{anyhow, Result};
use ethers::types::{Bytes, U256};
use minicbor::data::Type;
use minicbor::decode::{Decode, Decoder, Error as DecodeError};
use minicbor::encode::{Encode, Encoder, Error as EncodeError};
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
    fn bad_meta_map() -> Result<Self, DecodeError> {
        return Err(DecodeError::message("bad rain meta map"));
    }
    fn no_meta_map() -> Result<Self, DecodeError> {
        return Err(DecodeError::message("not rain meta map"));
    }
}

impl<'b> Decode<'b, ()> for RainMapDoc {
    fn decode(d: &mut Decoder<'b>, _: &mut ()) -> Result<Self, DecodeError> {
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

/// Receive a Rain Meta document with his prefix bytes and try to decode it usin cbor.
pub fn decode_rain_meta(meta_data: Bytes) -> Result<Vec<RainMapDoc>> {
    let (doc_magic_number, cbor_data) = meta_data.split_at(8);

    if MagicNumber::rain_meta_document_v1() == doc_magic_number.to_vec() {
        let mut decoder = Decoder::new(cbor_data);

        let mut all_docs: Vec<RainMapDoc> = vec![];

        while decoder.position() < decoder.input().len() {
            let doc: std::result::Result<RainMapDoc, DecodeError> = decoder.decode();

            if doc.is_err() {
                let errorsito = doc.unwrap_err();
                return Err(anyhow!("{}", errorsito.to_string()));
            }

            all_docs.push(doc.unwrap());
        }

        return Ok(all_docs);
    }
    return Err(anyhow!("Unable to decode - missing rain doc prefix"));
}

// pub fn decode_cbor(cbor_data: Vec<u8>) -> Result<Vec<RainMapDoc>> {
//     let mut decoder = Decoder::new(&cbor_data);

//     let mut all_docs: Vec<RainMapDoc> = vec![];

//     while decoder.position() < decoder.input().len() {
//         // TODO: Create error response
//         let doc: RainMapDoc = decoder.decode().unwrap();

//         all_docs.push(doc);
//     }

//     return Ok(all_docs);
// }

/// Receive a vec of RainMapDoc and try to encode it.
pub fn _encode_rain_meta(_docs: Vec<RainMapDoc>) -> Result<Vec<u8>> {
    //
    Ok([0].to_vec())
}
