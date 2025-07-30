use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use rain_orderbook_app_settings::{
    orderbook::OrderbookCfg,
    yaml::{
        orderbook::{OrderbookYaml as OrderbookYamlCfg, OrderbookYamlValidation},
        YamlError, YamlParsable,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct OrderbookYaml {
    yaml: OrderbookYamlCfg,
}

#[wasm_export]
impl OrderbookYaml {
    /// Creates a new OrderbookYaml instance from YAML configuration sources.
    ///
    /// This constructor parses one or more YAML configuration strings to create an OrderbookYaml
    /// instance that provides access to orderbook configurations, network settings, tokens, and
    /// other deployment metadata. The YAML sources are merged and validated according to the
    /// [orderbook specification](https://github.com/rainlanguage/specs/blob/main/ob-yaml.md).
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Basic usage with single YAML source
    /// const yamlConfig = `
    /// version: "4"
    /// networks:
    ///   mainnet:
    ///     rpc: https://mainnet.infura.io
    ///     chain-id: 1
    /// orderbooks:
    ///   my-orderbook:
    ///     address: 0x1234567890abcdef1234567890abcdef12345678
    ///     network: mainnet
    /// ...
    /// `;
    ///
    /// const result = OrderbookYaml.new([yamlConfig], false);
    /// if (result.error) {
    ///   console.error("Configuration error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orderbookYaml = result.value;
    /// // Do something with the orderbookYaml
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "Successfully parsed and configured instance"
    )]
    pub fn new(
        #[wasm_export(
            param_description = "Vector of YAML configuration strings to parse and merge"
        )]
        sources: Vec<String>,
        #[wasm_export(
            param_description = "Optional boolean to enable strict validation (defaults to false)"
        )]
        validate: Option<bool>,
    ) -> Result<OrderbookYaml, OrderbookYamlError> {
        let yaml = OrderbookYamlCfg::new(
            sources,
            match validate {
                Some(true) => OrderbookYamlValidation::full(),
                _ => OrderbookYamlValidation::default(),
            },
        )?;
        Ok(Self { yaml })
    }

    /// Retrieves orderbook configuration by its contract address from a parsed YAML configuration.
    ///
    /// This function looks up a specific orderbook configuration within a YAML configuration file
    /// using the orderbook's blockchain address. It's essential for accessing orderbook metadata
    /// including network configuration, subgraph endpoints, and other deployment details.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Basic usage
    /// const result = orderbookYaml.getOrderbookByAddress("0x1234567890abcdef1234567890abcdef12345678");
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orderbook = result.value;
    /// // Do something with the orderbook
    /// ```
    #[wasm_export(
        js_name = "getOrderbookByAddress",
        unchecked_return_type = "OrderbookCfg",
        return_description = "Complete orderbook configuration"
    )]
    pub fn get_orderbook_by_address(
        &self,
        #[wasm_export(param_description = "The hexadecimal address of the orderbook contract")]
        orderbook_address: &str,
    ) -> Result<OrderbookCfg, OrderbookYamlError> {
        let address =
            Address::from_str(orderbook_address).map_err(OrderbookYamlError::FromHexError)?;
        Ok(self.yaml.get_orderbook_by_address(address)?)
    }
}

#[derive(Error, Debug)]
pub enum OrderbookYamlError {
    #[error("Orderbook yaml error: {0}")]
    YamlError(#[from] YamlError),
    #[error("Invalid address: {0}")]
    FromHexError(#[from] FromHexError),
}

impl OrderbookYamlError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            OrderbookYamlError::YamlError(err) =>
                format!("There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"{}\"", err),
            OrderbookYamlError::FromHexError(err) =>
                format!("The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: \"{}\"", err),
        }
    }
}

impl From<OrderbookYamlError> for JsValue {
    fn from(value: OrderbookYamlError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
impl From<OrderbookYamlError> for WasmEncodedError {
    fn from(value: OrderbookYamlError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use wasm_bindgen_test::wasm_bindgen_test;

    pub fn get_yaml() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        secondary: https://api.thegraph.com/subgraphs/name/abc
    metaboards:
        board1: https://meta.example.com/board1
        board2: https://meta.example.com/board2
    orderbooks:
        orderbook1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
            subgraph: mainnet
            label: Primary Orderbook
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    deployers:
        deployer1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
    accounts:
        admin: 0x4567890123abcdef
        user: 0x5678901234abcdef
    sentry: true
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    fn test_orderbook_yaml() {
        let orderbook_yaml = OrderbookYaml::new(vec![get_yaml()], None).unwrap();
        let orderbook = orderbook_yaml
            .get_orderbook_by_address("0x0000000000000000000000000000000000000002")
            .unwrap();

        assert_eq!(
            orderbook.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(orderbook.key, "orderbook1");
        assert_eq!(orderbook.network.key, "mainnet");
        assert_eq!(orderbook.subgraph.key, "mainnet");
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_orderbook_yaml_error() {
        let orderbook_yaml = OrderbookYaml::new(vec![get_yaml()], None).unwrap();
        let orderbook = orderbook_yaml.get_orderbook_by_address("invalid-address");

        assert_eq!(orderbook.is_err(), true);
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_string(),
            "Invalid address: odd number of digits"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: \"odd number of digits\""
        );

        let orderbook =
            orderbook_yaml.get_orderbook_by_address("0x0000000000000000000000000000000000000000");
        assert_eq!(orderbook.is_err(), true);
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_string(),
            "Orderbook yaml error: orderbook with address: 0x0000000000000000000000000000000000000000 not found"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"orderbook with address: 0x0000000000000000000000000000000000000000 not found\""
        );
    }

    pub fn get_invalid_yaml() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    orderbooks:
        orderbook1:
            address: 0x0000000000000000000000000000000000000002
            network: nonexistent-network
            subgraph: nonexistent-subgraph
            label: Primary Orderbook
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    fn test_orderbook_yaml_invalid_with_validation_enabled() {
        let result = OrderbookYaml::new(vec![get_invalid_yaml()], Some(true));
        match result {
            Ok(_) => panic!("Expected validation error with invalid YAML"),
            Err(err) => {
                assert!(err.to_string().contains("Orderbook yaml error"));
                assert!(err
                    .to_readable_msg()
                    .contains("There was an error processing the YAML configuration"));
            }
        }
    }
}
