use dotrain::RainDocument;
use rain_orderbook_app_settings::{
    config::{Config, ParseConfigStringError},
    merge::MergeError,
    string_structs::ConfigString,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrontmatterError {
    #[error(transparent)]
    ParseConfigError(#[from] ParseConfigStringError),
    #[error(transparent)]
    MergeError(#[from] MergeError),
}

/// Parse dotrain frontmatter to extract Config
pub fn try_parse_frontmatter(frontmatter: &str) -> Result<Config, FrontmatterError> {
    if frontmatter.is_empty() {
        return Ok(Config::default());
    }
    Ok(frontmatter.try_into()?)
}

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn get_merged_config(
    dotrain: &str,
    top_config: Option<&str>,
) -> Result<Config, FrontmatterError> {
    let frontmatter = RainDocument::get_front_matter(dotrain).unwrap_or("");
    let mut frontmatter_str_config: ConfigString = frontmatter
        .try_into()
        .map_err(ParseConfigStringError::YamlDeserializerError)?;
    if let Some(v) = top_config {
        let top_str_config: ConfigString = v
            .try_into()
            .map_err(ParseConfigStringError::YamlDeserializerError)?;
        frontmatter_str_config.merge(top_str_config)?;
        Ok(frontmatter_str_config.try_into()?)
    } else {
        Ok(frontmatter_str_config.try_into()?)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_try_parse_rebinds() {
//         let frontmatter = "
// orderbook:
//     order:
//         deployer: 0x1111111111111111111111111111111111111111
//         valid-inputs:
//             - token: 0x0000000000000000000000000000000000000001
//               decimals: 18
//               vault-id: 0x1
//         valid-outputs:
//             - token: 0x0000000000000000000000000000000000000002
//               decimals: 18
//               vault-id: 0x2
// bind:
//     - some-binding: 12345
//     - some-other-binding: 2e16
//     - another-binding: \" some literal string \"
// ";
//         let rebinds = try_parse_frontmatter_rebinds(frontmatter).unwrap();
//         let expected = vec![
//             Rebind("some-binding".to_owned(), "12345".to_owned()),
//             Rebind("some-other-binding".to_owned(), "2e16".to_owned()),
//             Rebind(
//                 "another-binding".to_owned(),
//                 " some literal string ".to_owned(),
//             ),
//         ];
//         assert_eq!(rebinds, expected)
//     }

//     #[test]
//     fn test_try_parse_frontmatter() {
//         let frontmatter = "
// orderbook:
//     order:
//         deployer: 0x1111111111111111111111111111111111111111
//         valid-inputs:
//             - token: 0x0000000000000000000000000000000000000001
//               decimals: 18
//               vault-id: 0x1
//         valid-outputs:
//             - token: 0x0000000000000000000000000000000000000002
//               decimals: 18
//               vault-id: 0x2
// ";

//         let (deployer, valid_inputs, valid_outputs, _) =
//             try_parse_frontmatter(frontmatter).unwrap();

//         assert_eq!(
//             deployer,
//             "0x1111111111111111111111111111111111111111"
//                 .parse::<Address>()
//                 .unwrap()
//         );
//         assert_eq!(
//             valid_inputs[0].token,
//             "0x0000000000000000000000000000000000000001"
//                 .parse::<Address>()
//                 .unwrap()
//         );
//         assert_eq!(valid_inputs[0].decimals, 18);
//         assert_eq!(valid_inputs[0].vaultId, U256::from(1));
//         assert_eq!(
//             valid_outputs[0].token,
//             "0x0000000000000000000000000000000000000002"
//                 .parse::<Address>()
//                 .unwrap()
//         );
//         assert_eq!(valid_outputs[0].decimals, 18);
//         assert_eq!(valid_outputs[0].vaultId, U256::from(2));
//     }
// }
