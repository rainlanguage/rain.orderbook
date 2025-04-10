use alloy::primitives::BlockNumber;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify), tsify(namespace))]
pub enum BlockCfg {
    Number(#[cfg_attr(target_family = "wasm", tsify(type = "number"))] BlockNumber),
    Genesis,
    Latest,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BlockCfg);

impl BlockCfg {
    pub fn to_block_number(&self, latest_block: BlockNumber) -> BlockNumber {
        match self {
            BlockCfg::Number(n) => *n,
            BlockCfg::Genesis => 0,
            BlockCfg::Latest => latest_block,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct BlockRangeCfg {
    pub start: BlockCfg,
    pub end: BlockCfg,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BlockRangeCfg);

impl BlockRangeCfg {
    pub fn validate(&self, latest_block: BlockNumber) -> Result<(), BlockError> {
        let start = self.start.to_block_number(latest_block);
        let end = self.end.to_block_number(latest_block);
        if start > end {
            return Err(BlockError::InvalidBlockRange);
        }
        Ok(())
    }
}

// Serialize implementation for BlockRange
impl Serialize for BlockRangeCfg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut range_string = String::new();

        match &self.start {
            BlockCfg::Genesis => range_string.push_str(""),
            BlockCfg::Latest => range_string.push_str(""),
            BlockCfg::Number(n) => range_string.push_str(&n.to_string()),
        }

        range_string.push_str("..");

        match &self.end {
            BlockCfg::Genesis => range_string.push_str(""),
            BlockCfg::Latest => range_string.push_str(""),
            BlockCfg::Number(n) => range_string.push_str(&n.to_string()),
        }

        serializer.serialize_str(&range_string)
    }
}

// Deserialize implementation for BlockRange
impl<'de> Deserialize<'de> for BlockRangeCfg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BlockRangeVisitor)
    }
}

struct BlockRangeVisitor;

impl<'de> Visitor<'de> for BlockRangeVisitor {
    type Value = BlockRangeCfg;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a range in the form [a..b], [a..], or [..b]")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_range(v).map_err(de::Error::custom)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut range_string = String::new();

        while let Some(elem) = seq.next_element::<String>()? {
            range_string.push_str(&elem);
        }

        parse_range(&range_string).map_err(de::Error::custom)
    }
}

fn parse_range(s: &str) -> Result<BlockRangeCfg, String> {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() == 2 {
        let start = match parts[0] {
            "" => BlockCfg::Genesis,
            s => BlockCfg::Number(s.parse().map_err(|_| "Invalid block number")?),
        };
        let end = match parts[1] {
            "" => BlockCfg::Latest,
            s => BlockCfg::Number(s.parse().map_err(|_| "Invalid block number")?),
        };
        return Ok(BlockRangeCfg { start, end });
    }
    Err(format!("Invalid range syntax: {}", s))
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum BlocksCfg {
    RangeWithInterval { range: BlockRangeCfg, interval: u32 },
    SimpleRange(BlockRangeCfg),
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BlocksCfg);

#[derive(Debug, Error, PartialEq)]
pub enum BlockError {
    #[error("Invalid block range")]
    InvalidBlockRange,
}

impl BlocksCfg {
    pub fn expand_to_block_numbers(
        &self,
        latest_block: BlockNumber,
    ) -> Result<Vec<BlockNumber>, BlockError> {
        match self {
            BlocksCfg::RangeWithInterval { range, interval } => {
                range.validate(latest_block)?;
                let mut blocks = vec![];
                let mut current_block = range.start.to_block_number(latest_block);
                let end_block = range.end.to_block_number(latest_block);
                while current_block <= end_block {
                    blocks.push(current_block);
                    current_block += *interval as BlockNumber;
                }
                Ok(blocks)
            }
            BlocksCfg::SimpleRange(range) => {
                range.validate(latest_block)?;
                let start_block = range.start.to_block_number(latest_block);
                let end_block = range.end.to_block_number(latest_block);
                Ok((start_block..=end_block).collect())
            }
        }
    }
}

impl Default for BlocksCfg {
    fn default() -> Self {
        BlocksCfg::RangeWithInterval {
            range: BlockRangeCfg {
                start: BlockCfg::Genesis,
                end: BlockCfg::Latest,
            },
            interval: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_with_interval() {
        let yaml_data = r#"
range: [0..100]
interval: 5
"#;
        let expected = BlocksCfg::RangeWithInterval {
            range: BlockRangeCfg {
                start: BlockCfg::Number(0),
                end: BlockCfg::Number(100),
            },
            interval: 5,
        };

        let result: BlocksCfg = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = result.expand_to_block_numbers(100).unwrap();
        assert_eq!(
            expanded_blocks,
            vec![0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95, 100]
        );
    }

    #[test]
    fn test_simple_range() {
        let yaml_data = r#"
[0..100]
"#;
        let expected = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Number(0),
            end: BlockCfg::Number(100),
        });

        let result: BlocksCfg = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = result.expand_to_block_numbers(100).unwrap();
        assert_eq!(expanded_blocks, (0..=100).collect::<Vec<BlockNumber>>());
    }

    #[test]
    fn test_range_to_latest() {
        let yaml_data = r#"
[10..]
"#;
        let expected = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Number(10),
            end: BlockCfg::Latest,
        });

        let result: BlocksCfg = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = result.expand_to_block_numbers(20).unwrap();
        assert_eq!(expanded_blocks, (10..=20).collect::<Vec<BlockNumber>>());
    }

    #[test]
    fn test_range_from_genesis() {
        let yaml_data = r#"
[..50]
"#;
        let expected = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Genesis,
            end: BlockCfg::Number(50),
        });

        let result: BlocksCfg = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = result.expand_to_block_numbers(50).unwrap();
        assert_eq!(expanded_blocks, (0..=50).collect::<Vec<BlockNumber>>());
    }

    #[test]
    fn test_range_genesis_to_latest() {
        let yaml_data = r#"
[..]
"#;
        let expected = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Genesis,
            end: BlockCfg::Latest,
        });

        let result: BlocksCfg = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = result.expand_to_block_numbers(20).unwrap();
        assert_eq!(expanded_blocks, (0..=20).collect::<Vec<BlockNumber>>());
    }

    #[test]
    fn test_default_blocks() {
        let default_blocks = BlocksCfg::default();
        let expected = BlocksCfg::RangeWithInterval {
            range: BlockRangeCfg {
                start: BlockCfg::Genesis,
                end: BlockCfg::Latest,
            },
            interval: 1,
        };

        assert_eq!(default_blocks, expected);

        let serialized = serde_yaml::to_string(&default_blocks).unwrap();
        let deserialized: BlocksCfg = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);

        let expanded_blocks = default_blocks.expand_to_block_numbers(10).unwrap();
        assert_eq!(expanded_blocks, (0..=10).collect::<Vec<BlockNumber>>());
    }

    #[test]
    fn test_invalid_range() {
        let range = BlockRangeCfg {
            start: BlockCfg::Latest,
            end: BlockCfg::Genesis,
        };

        assert_eq!(range.validate(100), Err(BlockError::InvalidBlockRange));
    }

    #[test]
    fn test_to_block_number() {
        assert_eq!(BlockCfg::Genesis.to_block_number(100), 0);
        assert_eq!(BlockCfg::Latest.to_block_number(100), 100);
        assert_eq!(BlockCfg::Number(50).to_block_number(100), 50);
    }

    #[test]
    fn test_expand_to_block_numbers_invalid_range() {
        let blocks = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Latest,
            end: BlockCfg::Genesis,
        });

        assert_eq!(
            blocks.expand_to_block_numbers(100),
            Err(BlockError::InvalidBlockRange)
        );
    }

    #[test]
    fn test_expand_to_block_numbers_range_with_interval() {
        let blocks = BlocksCfg::RangeWithInterval {
            range: BlockRangeCfg {
                start: BlockCfg::Number(0),
                end: BlockCfg::Number(20),
            },
            interval: 5,
        };

        let expected = vec![0, 5, 10, 15, 20];
        assert_eq!(blocks.expand_to_block_numbers(100).unwrap(), expected);
    }

    #[test]
    fn test_expand_to_block_numbers_simple_range() {
        let blocks = BlocksCfg::SimpleRange(BlockRangeCfg {
            start: BlockCfg::Number(0),
            end: BlockCfg::Number(5),
        });

        let expected = vec![0, 1, 2, 3, 4, 5];
        assert_eq!(blocks.expand_to_block_numbers(100).unwrap(), expected);
    }
}
