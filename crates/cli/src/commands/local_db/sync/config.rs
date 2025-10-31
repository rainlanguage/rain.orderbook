use anyhow::{anyhow, Result};
use rain_orderbook_app_settings::{
    orderbook::OrderbookCfg,
    yaml::{
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlParsable,
    },
};

pub(crate) fn load_primary_orderbook_from_settings(
    chain_id: u32,
    settings_yaml: &str,
) -> Result<OrderbookCfg> {
    let orderbook_yaml = OrderbookYaml::new(
        vec![settings_yaml.to_owned()],
        OrderbookYamlValidation::default(),
    )
    .map_err(anyhow::Error::from)?;

    let orderbooks = orderbook_yaml
        .get_orderbooks_by_chain_id(chain_id)
        .map_err(anyhow::Error::from)?;

    // TODO: Support syncing multiple orderbooks for the same network in a single run.
    orderbooks
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No orderbooks configured for chain id {}", chain_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;

    const SAMPLE_SETTINGS: &str = r#"version: 2
networks:
  arbitrum:
    rpcs:
      - https://example.com
    chain-id: "42161"
    currency: ETH
subgraphs:
  arbitrum: https://example.com/subgraph
orderbooks:
  arbitrum:
    address: 0x550878091b2B1506069F61ae59e3A5484Bca9166
    network: arbitrum
    subgraph: arbitrum
    deployment-block: "123"
local-db-remotes:
  arbitrum: https://example.com/localdb/arbitrum
"#;

    #[test]
    fn load_orderbook_from_settings() {
        let orderbook =
            load_primary_orderbook_from_settings(42161, SAMPLE_SETTINGS).expect("orderbook");
        let expected_address =
            Address::from_str("0x550878091b2B1506069F61ae59e3A5484Bca9166").unwrap();
        assert_eq!(orderbook.address, expected_address);
        assert_eq!(orderbook.deployment_block, 123);
    }

    #[test]
    fn load_orderbook_missing_chain() {
        let error =
            load_primary_orderbook_from_settings(1, SAMPLE_SETTINGS).expect_err("missing chain");
        assert!(
            error.to_string().contains("chain-id: 1"),
            "unexpected error: {error}"
        );
    }
}
