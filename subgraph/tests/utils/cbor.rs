use super::{ascii_string_to_bytes, hash_keccak, MagicNumber};
use anyhow::{anyhow, Result};
use ethers::types::{Bytes, H256, U256};
use minicbor::data::Type;
use minicbor::decode::{Decode, Decoder, Error as DecodeError};
use minicbor::encode::{Encode, Encoder, Error as EncodeError, Write};
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
    /// Keys prensent on the RainDocument
    ///
    /// See: https://github.com/rainprotocol/specs/blob/main/metadata-v1.md#header-name-aliases-cbor-map-keys
    fn len(&self) -> usize {
        // Starting on two (2) since payload and magic_number are not optional.
        let mut count = 2;

        if self.content_type.is_some() {
            count += 1;
        }
        if self.content_encoding.is_some() {
            count += 1;
        }
        if self.content_language.is_some() {
            count += 1;
        }

        count
    }

    /// Length in bytes fo the rain document
    fn len_bytes(&self) -> usize {
        // The byte  used by the known length/size map
        let mut count = 1;

        // Getting the length of the payload
        // Key(1 byte) + string type(1 byte) + arg bytes + data bytes
        let payload_type_size = key_bytes_size(&self.payload);
        count += 2 + payload_type_size + &self.payload.len();

        // Tecnically, the magic number in RainDocument it should be a u64(8bytes)
        // So, it should 1 byte from the map key, 1 byte for the u64 representaiton
        // and 8 bytes for the number itself. So, in total is 10 bytes
        // Later we could add support for receiving any number.
        count += 10; // 10 bytes

        // Check for optional fiedls
        if self.content_type.is_some() {
            let string_bytes = ascii_string_to_bytes(self.content_type.clone().unwrap());
            let type_size = key_bytes_size(&string_bytes);

            // Key(1 byte) + string type(1 byte) + arg bytes + data bytes
            count += 2 + type_size + string_bytes.len();
        }
        if self.content_encoding.is_some() {
            let string_bytes = ascii_string_to_bytes(self.content_encoding.clone().unwrap());
            let type_size = key_bytes_size(&string_bytes);

            // Key(1 byte) + string type(1 byte) + arg bytes + data bytes
            count += 2 + type_size + string_bytes.len();
        }
        if self.content_language.is_some() {
            let string_bytes = ascii_string_to_bytes(self.content_language.clone().unwrap());
            let type_size = key_bytes_size(&string_bytes);

            // Key(1 byte) + string type(1 byte) + arg bytes + data bytes
            count += 2 + type_size + string_bytes.len();
        }

        // return it
        count
    }

    /// Hash the rain map document using Keccak256
    pub fn hash(&self) -> H256 {
        let doc_encoded = self.encode();

        hash_keccak(&doc_encoded)
    }

    /// CBOR encode the Rain Document using CBOR.
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0u8; self.len_bytes()];
        let mut encoder = Encoder::new(&mut buffer[..]);

        let _ = encoder.encode(self);

        return buffer;
    }

    fn bad_meta_map() -> Result<Self, DecodeError> {
        Err(DecodeError::message("bad rain meta map"))
    }
    fn no_meta_map() -> Result<Self, DecodeError> {
        Err(DecodeError::message("not rain meta map"))
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
            // Since it's starting to decode and it's not a map, return an error.
            Self::no_meta_map()
        }
    }
}

impl<C> Encode<C> for RainMapDoc {
    fn encode<W: Write>(
        &self,
        enc: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), EncodeError<W::Error>> {
        let doc_len = self.len() as u8;

        // Creating the map based on the rain document length
        let _ = enc.map(doc_len.into());

        // Key 0
        let _ = enc.u8(0);
        let _ = enc.bytes(&self.payload);

        // Key 1
        // Low_u64 to not panic (max u64 as the spec
        let _ = enc.u8(1);
        let _ = enc.u64(self.magic_number.low_u64());

        if self.content_type.is_some() {
            let _ = enc.u8(2);
            let _ = enc.str(&self.content_type.clone().unwrap());
        }

        if self.content_encoding.is_some() {
            let _ = enc.u8(3);
            let _ = enc.str(&self.content_encoding.clone().unwrap());
        }

        if self.content_language.is_some() {
            let _ = enc.u8(4);
            let _ = enc.str(&self.content_language.clone().unwrap());
        }

        Ok(())
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

    Err(anyhow!("Unable to decode - missing rain doc prefix"))
}

/// Receive a vec of RainMapDoc and try to encode it. If the length of the Vec is greater than one (1), then the output will be
/// an cbor sequence.
///
pub fn encode_rain_docs(docs: Vec<RainMapDoc>) -> Vec<u8> {
    let mut main_buffer = MagicNumber::rain_meta_document_v1().to_vec();

    for doc_index in 0..docs.len() {
        let doc = docs.get(doc_index).unwrap();

        let mut inner_buffer = doc.encode();

        main_buffer.append(&mut inner_buffer);
    }

    return main_buffer;
}

/// Based on the length of the bytes, return if it will need extra bytes to
/// store the byte length based in CBOR.
///
/// ## Example:
/// ```
/// // If size is 0, the bytes length will be in the args itself.
/// let size_0 = key_bytes_size(<<Bytes>>);
///
/// // If size is 1, it will need 1 byte more to store the length.
/// let size_1 = key_bytes_size(<<Bytes>>);
///
/// // If size is 2, it will need 2 byte more to store the length.
/// let size_2 = key_bytes_size(<<Bytes>>);
///
/// ```
fn key_bytes_size(bytes: &Bytes) -> usize {
    let size_bytes = bytes.len();

    if size_bytes < 119 {
        return 0;
    } else {
        let mut bytes = 1;
        let mut value = 255;
        while size_bytes > value {
            bytes *= 2;
            value = (value << 8) | 255;
        }
        bytes
    }
}
