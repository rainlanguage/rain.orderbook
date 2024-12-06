use alloy::primitives::Address;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{StrictYaml, StrictYamlLoader};

pub struct ObYaml {
    document: Arc<RwLock<StrictYaml>>,
}

impl ObYaml {
    pub fn new(yaml: &str) -> Self {
        let mut docs = StrictYamlLoader::load_from_str(yaml).unwrap();
        Self {
            document: Arc::new(RwLock::new(docs.pop().unwrap())),
        }
    }

    pub fn get_deployment_names(&self) -> Vec<String> {
        let doc = self.document.read().unwrap();
        let keys = doc["deployments"].as_hash().unwrap().keys();
        keys.map(|k| k.as_str().unwrap().to_string()).collect()
    }

    pub fn get_token(&self, key: &str) -> Arc<Token> {
        let doc = self.document.read().unwrap();
        let token = &doc["tokens"][key];
        Arc::new(Token {
            document: self.document.clone(),
            key: key.to_string(),
            network: token["network"].as_str().unwrap().to_string(),
            address: token["address"].as_str().unwrap().parse().unwrap(),
            decimals: token["decimals"].as_str().map(|s| s.to_string()),
            symbol: token["symbol"].as_str().map(|s| s.to_string()),
        })
    }

    pub fn get_order(&self, key: &str) -> Arc<Order> {
        let doc = self.document.read().unwrap();
        let order = &doc["orders"][key];
        Arc::new(Order {
            inputs: order["inputs"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|i| OrderIO {
                    token: self.get_token(i["token"].as_str().unwrap()),
                    vault_id: i["vault_id"].as_str().map(|s| s.to_string()),
                })
                .collect(),
            outputs: order["outputs"]
                .as_vec()
                .unwrap()
                .iter()
                .map(|i| OrderIO {
                    token: self.get_token(i["token"].as_str().unwrap()),
                    vault_id: i["vault_id"].as_str().map(|s| s.to_string()),
                })
                .collect(),
            network: order["network"].as_str().unwrap().to_string(),
            orderbook: order["orderbook"].as_str().map(|s| s.to_string()),
        })
    }

    pub fn get_deployment(&self, key: &str) -> Deployment {
        let doc = self.document.read().unwrap();
        let deployment = &doc["deployments"][key];
        Deployment {
            scenario: deployment["scenario"].as_str().unwrap().to_string(),
            order: self.get_order(deployment["order"].as_str().unwrap()),
        }
    }
}

pub struct Token {
    document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub network: String,
    pub address: Address,
    pub decimals: Option<String>,
    pub symbol: Option<String>,
}

impl Token {
    pub fn set_address(&self, address: Address) {
        if let Ok(mut doc) = self.document.write() {
            if let StrictYaml::Hash(ref mut hash) = *doc {
                if let Some(StrictYaml::Hash(ref mut tokens)) =
                    hash.get_mut(&StrictYaml::String("tokens".to_string()))
                {
                    if let Some(StrictYaml::Hash(ref mut token_hash)) =
                        tokens.get_mut(&StrictYaml::String(self.key.clone()))
                    {
                        token_hash.insert(
                            StrictYaml::String("address".to_string()),
                            StrictYaml::String(format!("{:#x}", address)),
                        );
                    }
                }
            }
        }
    }
}

pub struct Deployment {
    pub scenario: String,
    pub order: Arc<Order>,
}

pub struct OrderIO {
    pub token: Arc<Token>,
    pub vault_id: Option<String>,
}

pub struct Order {
    pub inputs: Vec<OrderIO>,
    pub outputs: Vec<OrderIO>,
    pub network: String,
    pub orderbook: Option<String>,
}

//tests
#[cfg(test)]
mod tests {
    use strict_yaml_rust::StrictYamlEmitter;

    use super::*;

    const YAML: &str = r#"
    # some comment
    tokens:
        usdc:
            network: arbitrum
            address: 0xaf88d065e77c8cc2239327c088bb5dee9d3f5f84
            decimals: 6
    orders:
        order1:
            network: arbitrum
            deployer: 0x1234567890123456789012345678901234567890
            orderbook: orderbook1
            inputs:
                - token: usdc
                  vault_id: 1
            outputs:
                - token: usdc
                  vault_id: 2
    deployments:
        deployment1:
            scenario: scenario1
            order: order1
    "#;

    #[test]
    fn test_get_deployment_names() {
        let yaml = ObYaml::new(YAML);
        assert_eq!(yaml.get_deployment_names(), vec!["deployment1"]);
    }

    #[test]
    fn test_get_token() {
        let yaml = ObYaml::new(YAML);
        let token = yaml.get_token("usdc");
        assert_eq!(
            token.address,
            "0xaf88d065e77c8cc2239327c088bb5dee9d3f5f84"
                .parse::<Address>()
                .unwrap()
        );
    }

    #[test]
    fn test_get_order() {
        let yaml = ObYaml::new(YAML);
        let order = yaml.get_order("order1");
        assert_eq!(order.inputs.len(), 1);
        assert_eq!(order.outputs.len(), 1);
    }

    #[test]
    fn test_get_deployment() {
        let yaml = ObYaml::new(YAML);
        let deployment = yaml.get_deployment("deployment1");
        assert_eq!(deployment.scenario, "scenario1");
        assert_eq!(deployment.order.inputs.len(), 1);
        assert_eq!(deployment.order.outputs.len(), 1);
        assert_eq!(
            deployment.order.inputs[0].token.address,
            "0xaf88d065e77c8cc2239327c088bb5dee9d3f5f84"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(deployment.order.network, "arbitrum");
    }

    #[test]
    fn test_modify_token_address() {
        let yaml = ObYaml::new(YAML);
        let deployment = yaml.get_deployment("deployment1");
        let token = deployment.order.inputs[0].token.clone();
        token.set_address(
            "0xaf88d065e77c8cc2239327c088bb5dee9d3f5f12"
                .parse::<Address>()
                .unwrap(),
        );
        assert_eq!(
            yaml.get_order("order1").inputs[0].token.address,
            "0xaf88d065e77c8cc2239327c088bb5dee9d3f5f12"
                .parse::<Address>()
                .unwrap()
        );
        let mut out_str = String::new();
        let mut emitter = StrictYamlEmitter::new(&mut out_str);
        emitter.compact(true);
        emitter.dump(&yaml.document.read().unwrap()).unwrap();

        println!("{}", out_str);

        let yaml2 = ObYaml::new(&out_str);
        let deployment = yaml2.get_deployment("deployment1");
        assert_eq!(
            deployment.order.inputs[0].token.address,
            "0xaf88d065e77c8cc2239327c088bb5dee9d3f5f12"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(deployment.order.network, "arbitrum");
        assert_eq!(deployment.order.inputs[0].vault_id, Some("1".to_string()));
    }
}
