use alloy_primitives::{Address, U256};
use dotrain::Rebind;
use rain_orderbook_bindings::IOrderBookV3::IO;
use strict_yaml_rust::{scanner::ScanError, StrictYaml, StrictYamlLoader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrontmatterError {
    #[error("frontmatter is not valid strict yaml: {0}")]
    FrontmatterInvalidYaml(#[from] ScanError),
    #[error("Invalid Field: {0}")]
    FrontmatterFieldInvalid(String),
    #[error("Missing Field: {0}")]
    FrontmatterFieldMissing(String),
    #[error("Frontmatter empty")]
    FrontmatterEmpty,
}

/// Parse dotrain frontmatter to extract deployer, valid-inputs and valid-outputs
#[allow(clippy::type_complexity)]
pub fn try_parse_frontmatter(
    frontmatter: &str,
) -> Result<(Address, Vec<IO>, Vec<IO>, Option<Vec<Rebind>>), FrontmatterError> {
    // Parse dotrain document frontmatter
    let frontmatter_yaml_vec = StrictYamlLoader::load_from_str(frontmatter)
        .map_err(FrontmatterError::FrontmatterInvalidYaml)?;
    let frontmatter_yaml = frontmatter_yaml_vec
        .get(0)
        .ok_or(FrontmatterError::FrontmatterEmpty)?;

    let deployer = frontmatter_yaml["orderbook"]["order"]["deployer"]
        .as_str()
        .ok_or(FrontmatterError::FrontmatterFieldMissing(
            "orderbook.order.deployer".into(),
        ))?
        .parse::<Address>()
        .map_err(|_| {
            FrontmatterError::FrontmatterFieldInvalid("orderbook.order.deployer".into())
        })?;

    let valid_inputs: Vec<IO> = try_parse_frontmatter_io(
        frontmatter_yaml["orderbook"]["order"]["valid-inputs"].clone(),
        "valid-inputs",
    )?;
    let valid_outputs: Vec<IO> = try_parse_frontmatter_io(
        frontmatter_yaml["orderbook"]["order"]["valid-outputs"].clone(),
        "valid-outputs",
    )?;

    let rebinds = get_rebinds_from_yaml(&frontmatter_yaml);

    Ok((deployer, valid_inputs, valid_outputs, rebinds))
}

/// parses a yaml text and tries to get rebindings from it
pub fn try_parse_frontmatter_rebinds(frontmatter: &str) -> Option<Vec<Rebind>> {
    let frontmatter_yaml_vec = StrictYamlLoader::load_from_str(frontmatter).ok()?;
    let frontmatter_yaml = frontmatter_yaml_vec.get(0)?;

    get_rebinds_from_yaml(&frontmatter_yaml)
}

/// gets rebindings from a parsed yaml
pub fn get_rebinds_from_yaml(frontmatter_yaml: &StrictYaml) -> Option<Vec<Rebind>> {
    let mut rebinds = vec![];
    let items = frontmatter_yaml["bind"].as_vec()?;
    for item in items {
        for (key, value) in item.as_hash()? {
            rebinds.push(Rebind(key.as_str()?.to_owned(), value.as_str()?.to_owned()))
        }
    }
    Some(rebinds)
}

/// Parse an Io array from from frontmatter field (i.e. valid-inputs or valid-outputs)
pub fn try_parse_frontmatter_io(
    io_yamls: StrictYaml,
    io_field_name: &str,
) -> Result<Vec<IO>, FrontmatterError> {
    io_yamls
        .into_vec()
        .ok_or(FrontmatterError::FrontmatterFieldMissing(format!(
            "orderbook.order.{}",
            io_field_name
        )))?
        .into_iter()
        .map(|io_yaml| -> Result<IO, FrontmatterError> {
            Ok(IO {
                token: io_yaml["token"]
                    .as_str()
                    .ok_or(FrontmatterError::FrontmatterFieldMissing(format!(
                        "orderbook.order.{}.token",
                        io_field_name
                    )))?
                    .parse::<Address>()
                    .map_err(|_| {
                        FrontmatterError::FrontmatterFieldInvalid(format!(
                            "orderbook.order.{}.token",
                            io_field_name
                        ))
                    })?,
                decimals: io_yaml["decimals"]
                    .as_str()
                    .ok_or(FrontmatterError::FrontmatterFieldMissing(format!(
                        "orderbook.order.{}.decimals",
                        io_field_name
                    )))?
                    .parse::<u8>()
                    .map_err(|_| {
                        FrontmatterError::FrontmatterFieldInvalid(format!(
                            "orderbook.order.{}.decimals",
                            io_field_name
                        ))
                    })?,
                vaultId: io_yaml["vault-id"]
                    .as_str()
                    .ok_or(FrontmatterError::FrontmatterFieldMissing(format!(
                        "orderbook.order.{}.vault-id",
                        io_field_name
                    )))?
                    .parse::<U256>()
                    .map_err(|_| {
                        FrontmatterError::FrontmatterFieldInvalid(format!(
                            "orderbook.order.{}.vault-id",
                            io_field_name
                        ))
                    })?,
            })
        })
        .collect::<Result<Vec<IO>, FrontmatterError>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_rebinds() {
        let frontmatter = "
orderbook:
    order:
        deployer: 0x1111111111111111111111111111111111111111
        valid-inputs:
            - token: 0x0000000000000000000000000000000000000001
              decimals: 18
              vault-id: 0x1
        valid-outputs:
            - token: 0x0000000000000000000000000000000000000002
              decimals: 18
              vault-id: 0x2
bind:
    - some-binding: 12345
    - some-other-binding: 2e16
    - another-binding: \" some literal string \"
";
        let rebinds = try_parse_frontmatter_rebinds(frontmatter).unwrap();
        let expected = vec![
            Rebind("some-binding".to_owned(), "12345".to_owned()),
            Rebind("some-other-binding".to_owned(), "2e16".to_owned()),
            Rebind(
                "another-binding".to_owned(),
                " some literal string ".to_owned(),
            ),
        ];
        assert_eq!(rebinds, expected)
    }

    #[test]
    fn test_try_parse_frontmatter() {
        let frontmatter = "
orderbook:
    order:
        deployer: 0x1111111111111111111111111111111111111111
        valid-inputs:
            - token: 0x0000000000000000000000000000000000000001
              decimals: 18
              vault-id: 0x1
        valid-outputs:
            - token: 0x0000000000000000000000000000000000000002
              decimals: 18
              vault-id: 0x2
";

        let (deployer, valid_inputs, valid_outputs, _) =
            try_parse_frontmatter(frontmatter).unwrap();

        assert_eq!(
            deployer,
            "0x1111111111111111111111111111111111111111"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            valid_inputs[0].token,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(valid_inputs[0].decimals, 18);
        assert_eq!(valid_inputs[0].vaultId, U256::from(1));
        assert_eq!(
            valid_outputs[0].token,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(valid_outputs[0].decimals, 18);
        assert_eq!(valid_outputs[0].vaultId, U256::from(2));
    }
}
