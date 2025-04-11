use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg,
    order::OrderCfg,
    orderbook::OrderbookCfg,
    yaml::{
        dotrain::DotrainYaml as DotrainYamlCfg, orderbook::OrderbookYaml as OrderbookYamlCfg,
        YamlError, YamlParsable,
    },
};
use rain_orderbook_common::dotrain::RainDocument;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct OrderbookYaml {
    yaml: Vec<String>,
}
impl PartialEq for OrderbookYaml {
    fn eq(&self, other: &Self) -> bool {
        self.yaml == other.yaml
    }
}

impl OrderbookYaml {
    fn get_orderbook_yaml_cfg(&self) -> Result<OrderbookYamlCfg, OrderbookYamlError> {
        Ok(OrderbookYamlCfg::new(self.yaml.clone(), false)?)
    }

    fn get_dotrain_yaml_cfg(&self) -> Result<DotrainYamlCfg, OrderbookYamlError> {
        Ok(DotrainYamlCfg::new(self.yaml.clone(), false)?)
    }
}

#[wasm_bindgen]
impl OrderbookYaml {
    #[wasm_bindgen(constructor)]
    pub fn new(yaml: Vec<String>) -> Result<Self, OrderbookYamlError> {
        let frontmatters = yaml
            .iter()
            .map(|yaml| {
                let frontmatter = RainDocument::get_front_matter(yaml)
                    .unwrap_or("")
                    .to_string();
                if frontmatter.is_empty() {
                    yaml.clone()
                } else {
                    frontmatter
                }
            })
            .collect();
        Ok(Self { yaml: frontmatters })
    }
}

#[wasm_export]
impl OrderbookYaml {
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
        let orderbook_yaml = self.get_orderbook_yaml_cfg()?;
        Ok(orderbook_yaml.get_orderbook_by_address(address)?)
    }

    #[wasm_export(js_name = "getOrderbookByDeploymentKey")]
    pub fn get_orderbook_by_deployment_key(
        &self,
        deployment_key: &str,
    ) -> Result<OrderbookCfg, OrderbookYamlError> {
        let orderbook_yaml = self.get_orderbook_yaml_cfg()?;
        let dotrain_yaml = self.get_dotrain_yaml_cfg()?;

        let order_key =
            DeploymentCfg::parse_order_key(dotrain_yaml.documents.clone(), deployment_key)?;
        let orderbook_key =
            OrderCfg::parse_orderbook_key(dotrain_yaml.documents.clone(), &order_key)?;

        Ok(orderbook_yaml.get_orderbook(&orderbook_key)?)
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
    use wasm_bindgen_test::wasm_bindgen_test;

    const FULL_YAML: &str = r#"
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
    scenarios:
        scenario1:
            deployer: deployer1
            runs: 1
    orders:
        order1:
            orderbook: orderbook1
            inputs:
                - token: token1
            outputs:
                - token: token1
        order2:
            inputs:
                - token: token1
            outputs:
                - token: token1
    deployments:
        deployment1:
            order: order1
            scenario: scenario1
        deployment2:
            order: order2
            scenario: scenario1
    deployers:
        deployer1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
    accounts:
        admin: 0x4567890123abcdef
        user: 0x5678901234abcdef
    sentry: true
    raindex-version: 1.0.0
    "#;

    const RAINLANG: &str = r#"
    ---
    #calculate-io
    max-output: max-value(),
    io: 1;
    #handle-io
    :;
    #handle-add-order
    :;
    "#;

    #[wasm_bindgen_test]
    fn test_get_orderbook_by_address() {
        let orderbook_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()]).unwrap();
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
    fn test_get_orderbook_by_address_error() {
        let orderbook_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()]).unwrap();

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

    #[wasm_bindgen_test]
    fn test_get_orderbook_by_deployment_key() {
        let orderbook_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()]).unwrap();
        let orderbook = orderbook_yaml
            .get_orderbook_by_deployment_key("deployment1")
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
    fn test_get_orderbook_by_deployment_key_error() {
        let orderbook_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()]).unwrap();

        let orderbook = orderbook_yaml.get_orderbook_by_deployment_key("deployment2");
        assert_eq!(orderbook.is_err(), true);
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_string(),
            "Orderbook yaml error: Orderbook key not found for order: order2"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"Orderbook key not found for order: order2\""
        );

        let orderbook = orderbook_yaml.get_orderbook_by_deployment_key("deployment3");
        assert_eq!(orderbook.is_err(), true);
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_string(),
            "Orderbook yaml error: Missing required field 'order' in deployment 'deployment3'"
        );
        assert_eq!(
            orderbook.as_ref().err().unwrap().to_readable_msg(),
            "There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"Missing required field 'order' in deployment 'deployment3'\""
        );
    }

    #[wasm_bindgen_test]
    fn test_work_with_dotrain() {
        let dotrain = format!("{}\n{}", FULL_YAML, RAINLANG);
        let orderbook_yaml = OrderbookYaml::new(vec![dotrain]).unwrap();
        let orderbook = orderbook_yaml
            .get_orderbook_by_deployment_key("deployment1")
            .unwrap();
        assert_eq!(orderbook.key, "orderbook1");
    }
}
