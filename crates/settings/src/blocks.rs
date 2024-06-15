use alloy_primitives::BlockNumber;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Clone)]
pub enum Block {
    Number(BlockNumber),
    Genesis,
    Latest,
}

#[typeshare]
#[derive(Debug, PartialEq, Clone)]
pub struct BlockRange {
    start: Block,
    end: Block,
}

// Serialize implementation for BlockRange
impl Serialize for BlockRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut range_string = String::new();

        match &self.start {
            Block::Genesis => range_string.push_str("genesis"),
            Block::Latest => range_string.push_str("latest"),
            Block::Number(n) => range_string.push_str(&n.to_string()),
        }

        range_string.push_str("..");

        match &self.end {
            Block::Genesis => range_string.push_str("genesis"),
            Block::Latest => range_string.push_str("latest"),
            Block::Number(n) => range_string.push_str(&n.to_string()),
        }

        serializer.serialize_str(&range_string)
    }
}

// Deserialize implementation for BlockRange
impl<'de> Deserialize<'de> for BlockRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BlockRangeVisitor)
    }
}

struct BlockRangeVisitor;

impl<'de> Visitor<'de> for BlockRangeVisitor {
    type Value = BlockRange;

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
            println!("elem: {}", elem);
            range_string.push_str(&elem);
        }

        parse_range(&range_string).map_err(de::Error::custom)
    }
}

fn parse_range(s: &str) -> Result<BlockRange, String> {
    let parts: Vec<&str> = s.split("..").collect();
    if parts.len() == 2 {
        let start = match parts[0] {
            "" => Block::Genesis,
            s => Block::Number(s.parse().map_err(|_| "Invalid block number")?),
        };
        let end = match parts[1] {
            "" => Block::Latest,
            s => Block::Number(s.parse().map_err(|_| "Invalid block number")?),
        };
        return Ok(BlockRange { start, end });
    }
    Err(format!("Invalid range syntax: {}", s))
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[typeshare]
pub enum Blocks {
    RangeWithInterval { range: BlockRange, interval: u32 },
    SimpleRange(BlockRange),
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks::RangeWithInterval {
            range: BlockRange {
                start: Block::Genesis,
                end: Block::Latest,
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
        let expected = Blocks::RangeWithInterval {
            range: BlockRange {
                start: Block::Number(0),
                end: Block::Number(100),
            },
            interval: 5,
        };

        let result: Blocks = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: Blocks = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_simple_range() {
        let yaml_data = r#"
[0..100]
"#;
        let expected = BlockRange {
            start: Block::Number(0),
            end: Block::Number(100),
        };

        let result: BlockRange = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: BlockRange = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_range_to_latest() {
        let yaml_data = r#"
[10..]
"#;
        let expected = Blocks::SimpleRange(BlockRange {
            start: Block::Number(10),
            end: Block::Latest,
        });

        let result: Blocks = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: Blocks = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_range_from_genesis() {
        let yaml_data = r#"
[..50]
"#;
        let expected = Blocks::SimpleRange(BlockRange {
            start: Block::Genesis,
            end: Block::Number(50),
        });

        let result: Blocks = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: Blocks = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_range_genesis_to_latest() {
        let yaml_data = r#"
[..]
"#;
        let expected = Blocks::SimpleRange(BlockRange {
            start: Block::Genesis,
            end: Block::Latest,
        });

        let result: Blocks = serde_yaml::from_str(yaml_data).unwrap();
        assert_eq!(result, expected);

        let serialized = serde_yaml::to_string(&result).unwrap();
        let deserialized: Blocks = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_default_blocks() {
        let default_blocks = Blocks::default();
        let expected = Blocks::RangeWithInterval {
            range: BlockRange {
                start: Block::Genesis,
                end: Block::Latest,
            },
            interval: 1,
        };

        assert_eq!(default_blocks, expected);

        let serialized = serde_yaml::to_string(&default_blocks).unwrap();
        let deserialized: Blocks = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, expected);
    }
}
