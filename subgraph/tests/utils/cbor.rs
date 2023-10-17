use super::MagicNumber;
use anyhow::{anyhow, Error, Result};
use ethers::types::{Bytes, U256};
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

// impl<C> Encode<C> for RainMapDoc {
//     fn encode<W: Write>(
//         &self,
//         enc: &mut Encoder<W>,
//         ctx: &mut C,
//     ) -> Result<(), EncodeError<W::Error>> {
//         println!("&self: {:?}\n", &self);

//         let doc_len = &self.len();
//         println!("doc_len: {:?}\n", doc_len);

//         enc.u8(20)?.end();
//         // enc.map(1)

//         // println!("xdd: {:?}", pave);
//         // let averr = pave.ok();
//         // if pave.is_err() {
//         //     println!("pave failed");
//         // } else {
//         //     println!("pave is ok");
//         // }

//         Ok(())
//     }
// }

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

/// Receive a vec of RainMapDoc and try to encode it.
///
/// **NOTE:** If the length of the Vec is greater than one (1), then the output will be
/// an cbor sequence.
pub fn _encode_rain_meta(docs: Vec<RainMapDoc>) -> Result<Vec<u8>> {
    let cbor_items: usize = docs.len();
    println!("cbor_items: {}", cbor_items);

    let mut main_buffer: Vec<u8> = Vec::new();
    let mut buffer = [0u8; 4];
    // let mut buffer = [0u8; 128];

    let mut encoder = Encoder::new(&mut buffer[..]);

    let aver = encoder.map(1).unwrap().u8(0).unwrap().u8(200).unwrap();

    println!("xd_0: {}", aver.writer().len());
    aver.writer_mut().fill(99u8);

    println!("xd_1: {}", aver.writer().len());

    println!("buffer: {}", Bytes::from(buffer));

    // let mut main_buffer: Vec<u8> = Vec::new();
    // // let mut encoder: Encoder<&mut [u8]> = Encoder::new(&mut main_buffer);

    // for doc_index in 0..cbor_items {
    //     let mut buffer = [0u8; 128];

    //     let mut encoder = Encoder::new(&mut buffer[..]);

    //     let doc = docs.get(doc_index).unwrap();
    //     let doc_len = doc.len() as u8;

    //     // Creating the map based on the rain document length
    //     encoder.map(doc_len.into()).unwrap();

    //     for key in 0..doc_len {
    //         match key {
    //             0 => {
    //                 //
    //                 encoder.u8(key);
    //             }

    //             // 1 => magic_number = Some(d.u64()?.into()),

    //             // 2 => content_type = Some(d.str()?.to_string()),

    //             // 3 => content_encoding = Some(d.str()?.to_string()),

    //             // 4 => content_language = Some(d.str()?.to_string()),

    //             // Does not allow other keys than the defnied by the metadata spec.
    //             // See: https://github.com/rainprotocol/specs/blob/main/metadata-v1.md#header-name-aliases-cbor-map-keys
    //             _ => {
    //                 //
    //             }
    //         }
    //     }

    //     //
    // }

    // let mut buffer = Vec::new();
    // let mut encoder: Encoder<&mut [u8]> = Encoder::new(&mut buffer);

    // // Iterate over your data chunks and encode them
    // for chunk in data_chunks {
    //     encoder.encode(chunk)?;
    // }

    // let response = encoder.encode(single_doc);

    // println!("buffer_end: {}", Bytes::from(buffer));

    // encoder.

    // Encoder::encode(&mut self, single_doc);

    // let single_doc_0 = docs.get(0).unwrap();
    // let size_0: usize = single_doc_0.len();
    // println!("size_0: {}", size_0);

    // let single_doc_1 = docs.get(1).unwrap();
    // let size_1: usize = single_doc_1.len();
    // println!("size_1: {}", size_1);
    // Encoder::map(&mut self, len)

    // for _ in 0..cbor_items {
    //     //
    // }

    Ok([0].to_vec())
}

// TODO: Use this for recursive encode the RainDocs
//
// let mut data: [u8; 1024] = [1; 1024]; // Example filled array
//
// data[data.len() - 1] = 0;
// data[data.len() - 2] = 0;
// println!("1: {:?}", data.len());
//
// let resp = remove_trailing_zeros(&data);
// println!("2: {:?}", resp.unwrap().len());
fn _remove_trailing_zeros(arr: &[u8]) -> Option<Vec<u8>> {
    // Find the position of the last non-zero element
    let length = arr.iter().rposition(|&x| x != 0).map(|pos| pos + 1);

    match length {
        Some(len) => {
            // Create a new Vec<u8> with the non-zero data
            let new_vec: Vec<u8> = arr[0..len].to_vec();
            Some(new_vec)
        }
        None => None, // All elements are zeros
    }
}
