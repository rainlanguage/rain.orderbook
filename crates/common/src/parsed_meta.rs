use rain_metadata::{
    types::dotrain::{gui_state_v1::DotrainGuiStateV1, source_v1::DotrainSourceV1},
    KnownMagic, RainMetaDocumentV1Item,
};
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use tsify::Tsify;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum ParsedMeta {
    DotrainGuiStateV1(DotrainGuiStateV1),
    DotrainSourceV1(DotrainSourceV1),
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ParsedMeta);

impl ParsedMeta {
    /// Parse metadata directly from RainMetaDocumentV1Item
    /// Returns Some(ParsedMeta) if the item is a type we need for the frontend,
    /// None otherwise (filtering out unnecessary metadata types)
    /// Unsupported meta types will be ignored
    pub fn from_meta_item(
        item: &RainMetaDocumentV1Item,
    ) -> Result<Option<Self>, rain_metadata::Error> {
        match item.magic {
            KnownMagic::DotrainGuiStateV1 => {
                let gui_state = DotrainGuiStateV1::try_from(item.clone())?;
                Ok(Some(ParsedMeta::DotrainGuiStateV1(gui_state)))
            }
            KnownMagic::DotrainSourceV1 => {
                let source = DotrainSourceV1::try_from(item.clone())?;
                Ok(Some(ParsedMeta::DotrainSourceV1(source)))
            }
            // Filter out all other metadata types - they're not needed for the frontend
            _ => Ok(None),
        }
    }

    /// Parse multiple metadata items from a vector of RainMetaDocumentV1Item
    /// Returns only the items relevant to the orderbook frontend
    pub fn parse_multiple(
        items: &[RainMetaDocumentV1Item],
    ) -> Result<Vec<Self>, rain_metadata::Error> {
        items
            .iter()
            .map(Self::from_meta_item)
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    /// Parse metadata from raw bytes
    /// This method parses the complete metadata document and extracts only orderbook-relevant items
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<Vec<Self>, rain_metadata::Error> {
        let items = RainMetaDocumentV1Item::cbor_decode(bytes)?;
        Self::parse_multiple(&items)
    }
}

#[cfg(test)]
mod tests {
    use alloy::{hex::FromHex, primitives::Address};
    use rain_metadata::types::dotrain::gui_state_v1::{ShortenedTokenCfg, ValueCfg};

    use super::*;
    use std::collections::BTreeMap;

    fn get_default_dotrain_source() -> DotrainSourceV1 {
        DotrainSourceV1("default source".to_string())
    }
    fn get_default_dotrain_gui_state() -> DotrainGuiStateV1 {
        DotrainGuiStateV1 {
            dotrain_hash: get_default_dotrain_source().hash(),
            field_values: BTreeMap::from([(
                "field1".to_string(),
                ValueCfg {
                    id: "field_id1".to_string(),
                    name: None,
                    value: "100".to_string(),
                },
            )]),
            deposits: BTreeMap::from([(
                "deposit1".to_string(),
                ValueCfg {
                    id: "deposit_id1".to_string(),
                    name: None,
                    value: "200".to_string(),
                },
            )]),
            select_tokens: BTreeMap::from([(
                "token1".to_string(),
                ShortenedTokenCfg {
                    network: "mainnet".to_string(),
                    address: Address::from_hex("0x1234567890123456789012345678901234567890")
                        .unwrap(),
                },
            )]),
            vault_ids: BTreeMap::from([("vault1".to_string(), Some("value4".to_string()))]),
            selected_deployment: "1".to_string(),
        }
    }

    #[test]
    fn test_from_meta_item_gui_state_v1() {
        let gui_state = get_default_dotrain_gui_state();
        let item = RainMetaDocumentV1Item::try_from(gui_state.clone()).unwrap();
        let result = ParsedMeta::from_meta_item(&item).unwrap();
        match result.unwrap() {
            ParsedMeta::DotrainGuiStateV1(parsed_gui_state) => {
                assert_eq!(parsed_gui_state, gui_state);
            }
            _ => panic!("Expected DotrainGuiStateV1"),
        }
    }

    #[test]
    fn test_from_meta_item_source_v1() {
        let source = get_default_dotrain_source();
        let item = RainMetaDocumentV1Item::from(source.clone());
        let result = ParsedMeta::from_meta_item(&item).unwrap();
        assert!(result.is_some());

        match result.unwrap() {
            ParsedMeta::DotrainSourceV1(parsed_source) => {
                assert_eq!(parsed_source.0, source.0);
            }
            _ => panic!("Expected DotrainSourceV1"),
        }
    }

    #[test]
    fn test_parse_multiple_items() {
        let gui_state = get_default_dotrain_gui_state();
        let source = get_default_dotrain_source();

        let items = vec![
            RainMetaDocumentV1Item::try_from(gui_state.clone()).unwrap(),
            RainMetaDocumentV1Item::from(source.clone()),
        ];

        let results = ParsedMeta::parse_multiple(&items).unwrap();
        assert_eq!(results.len(), 2);

        match &results[0] {
            ParsedMeta::DotrainGuiStateV1(parsed_gui_state) => {
                assert_eq!(*parsed_gui_state, gui_state);
            }
            _ => panic!("Expected DotrainGuiStateV1"),
        }

        match &results[1] {
            ParsedMeta::DotrainSourceV1(parsed_source) => {
                assert_eq!(parsed_source.0, source.0);
            }
            _ => panic!("Expected DotrainSourceV1"),
        }
    }

    #[test]
    fn test_parse_multiple_empty_vector() {
        let items = vec![];
        let results = ParsedMeta::parse_multiple(&items).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_from_bytes_invalid() {
        let invalid_bytes = b"invalid cbor data";
        let result = ParsedMeta::parse_from_bytes(invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_from_bytes_valid() {
        // Arrange two items
        let source = DotrainSourceV1("hello".to_string());
        let gui = DotrainGuiStateV1 {
            dotrain_hash: source.hash(),
            field_values: std::collections::BTreeMap::new(),
            deposits: std::collections::BTreeMap::new(),
            select_tokens: std::collections::BTreeMap::new(),
            vault_ids: std::collections::BTreeMap::new(),
            selected_deployment: "d".to_string(),
        };
        let items = vec![
            RainMetaDocumentV1Item::from(source.clone()),
            RainMetaDocumentV1Item::try_from(gui.clone()).unwrap(),
        ];
        let bytes = RainMetaDocumentV1Item::cbor_encode_seq(&items, KnownMagic::RainMetaDocumentV1)
            .unwrap();

        // Act
        let parsed = ParsedMeta::parse_from_bytes(&bytes).unwrap();

        // Assert
        assert!(matches!(&parsed[0], ParsedMeta::DotrainSourceV1(s) if s.0 == source.0));
        assert!(matches!(&parsed[1], ParsedMeta::DotrainGuiStateV1(g) if g == &gui));
    }
}
