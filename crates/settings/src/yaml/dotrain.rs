use super::*;
use crate::{Order, Scenario};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DotrainYaml {
    pub document: Arc<RwLock<StrictYaml>>,
}

impl YamlParsable for DotrainYaml {
    fn new(source: String, validate: bool) -> Result<Self, YamlError> {
        let docs = StrictYamlLoader::load_from_str(&source)?;
        if docs.is_empty() {
            return Err(YamlError::EmptyFile);
        }
        let doc = docs[0].clone();
        let document = Arc::new(RwLock::new(doc));

        if validate {
            Order::parse_all_from_yaml(document.clone())?;
        }

        Ok(DotrainYaml { document })
    }
}

impl DotrainYaml {
    pub fn get_order_keys(&self) -> Result<Vec<String>, YamlError> {
        let orders = Order::parse_all_from_yaml(self.document.clone())?;
        Ok(orders.keys().cloned().collect())
    }
    pub fn get_order(&self, key: &str) -> Result<Order, YamlError> {
        Order::parse_from_yaml(self.document.clone(), key)
    }

    pub fn get_scenario_keys(&self) -> Result<Vec<String>, YamlError> {
        let scenarios = Scenario::parse_all_from_yaml(self.document.clone())?;
        Ok(scenarios.keys().cloned().collect())
    }
    pub fn get_scenario(&self, key: &str) -> Result<Scenario, YamlError> {
        Scenario::parse_from_yaml(self.document.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::U256;
    use orderbook::OrderbookYaml;

    use super::*;

    const FULL_YAML: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpc: https://testnet.infura.io
            chain-id: 1337
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
        token2:
            network: mainnet
            address: 0x0000000000000000000000000000000000000002
            decimals: 6
            label: USD Coin
            symbol: USDC
    deployers:
        deployer1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
        deployer2:
            address: 0x0000000000000000000000000000000000000003
            network: testnet
    orders:
        order1:
            inputs:
                - token: token1
                  vault-id: 1
            outputs:
                - token: token2
                  vault-id: 2
    scenarios:
        scenario1:
            bindings:
                key1: value1
            deployer: deployer1
            scenarios:
                scenario2:
                    bindings:
                        key2: value2
    "#;

    #[test]
    fn test_full_yaml() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();
        let dotrain_yaml = DotrainYaml::new(FULL_YAML.to_string(), false).unwrap();

        assert_eq!(dotrain_yaml.get_order_keys().unwrap().len(), 1);
        let order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        let input = order.inputs.first().unwrap();
        assert_eq!(
            *input.token.clone().as_ref(),
            ob_yaml.get_token("token1").unwrap()
        );
        assert_eq!(input.vault_id, Some(U256::from(1)));
        let output = order.outputs.first().unwrap();
        assert_eq!(*output.token.as_ref(), ob_yaml.get_token("token2").unwrap());
        assert_eq!(output.vault_id, Some(U256::from(2)));
        assert_eq!(
            *order.network.as_ref(),
            ob_yaml.get_network("mainnet").unwrap()
        );

        let scenario_keys = dotrain_yaml.get_scenario_keys().unwrap();
        assert_eq!(scenario_keys.len(), 2);
        let scenario1 = dotrain_yaml.get_scenario("scenario1").unwrap();
        assert_eq!(scenario1.bindings.len(), 1);
        assert_eq!(scenario1.bindings.get("key1").unwrap(), "value1");
        assert_eq!(
            *scenario1.deployer.as_ref(),
            ob_yaml.get_deployer("deployer1").unwrap()
        );
        let scenario2 = dotrain_yaml.get_scenario("scenario2").unwrap();
        assert_eq!(scenario2.bindings.len(), 2);
        assert_eq!(scenario2.bindings.get("key1").unwrap(), "value1");
        assert_eq!(scenario2.bindings.get("key2").unwrap(), "value2");
        assert_eq!(
            *scenario2.deployer.as_ref(),
            ob_yaml.get_deployer("deployer2").unwrap()
        );
    }
}
