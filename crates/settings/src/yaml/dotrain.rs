use super::*;
use crate::Order;
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
    orders:
        order1:
            inputs:
                - token: token1
                  vault-id: 1
            outputs:
                - token: token2
                  vault-id: 2
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
    }
}
