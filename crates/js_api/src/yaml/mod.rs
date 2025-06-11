use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use rain_orderbook_app_settings::{
    orderbook::OrderbookCfg,
    yaml::{orderbook::OrderbookYaml as OrderbookYamlCfg, YamlError, YamlParsable},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct OrderbookYaml {
    yaml: OrderbookYamlCfg,
}
impl PartialEq for OrderbookYaml {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.yaml, &other.yaml)
    }
}

#[wasm_export]
impl OrderbookYaml {
    #[wasm_export(js_name = "new", preserve_js_class)]
    pub fn new(
        sources: Vec<String>,
        validate: Option<bool>,
    ) -> Result<OrderbookYaml, OrderbookYamlError> {
        let yaml = OrderbookYamlCfg::new(sources, validate.unwrap_or(false))?;
        Ok(Self { yaml })
    }

    #[wasm_export(
        js_name = "getOrderbookByAddress",
        unchecked_return_type = "OrderbookCfg"
    )]
    pub fn get_orderbook_by_address(
        &self,
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
            rpc: https://mainnet.infura.io
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
            "Invalid address: Odd number of digits"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: \"Odd number of digits\""
        );

        let orderbook =
            orderbook_yaml.get_orderbook_by_address("0x0000000000000000000000000000000000000000");
        assert_eq!(orderbook.is_err(), true);
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_string(),
            "Orderbook yaml error: Key '0x0000000000000000000000000000000000000000' not found"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"Key '0x0000000000000000000000000000000000000000' not found\""
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

    #[wasm_bindgen_test]
    fn test_orderbook_yaml_invalid_with_validation_disabled() {
        let result = OrderbookYaml::new(vec![get_invalid_yaml()], Some(false));
        if let Err(err) = result {
            assert!(err.to_string().contains("Orderbook yaml error"));
            assert!(err
                .to_readable_msg()
                .contains("There was an error processing the YAML configuration"));
        }
    }
}
