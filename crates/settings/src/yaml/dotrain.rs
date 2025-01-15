use super::*;
use crate::{Deployment, Gui, Order, Scenario};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt,
    sync::{Arc, RwLock},
};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[derive(Debug, Clone, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DotrainYaml {
    pub documents: Vec<Arc<RwLock<StrictYaml>>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(DotrainYaml);

impl YamlParsable for DotrainYaml {
    fn new(sources: Vec<String>, validate: bool) -> Result<Self, YamlError> {
        let mut documents = Vec::new();

        for source in sources {
            let docs = StrictYamlLoader::load_from_str(&source)?;
            if docs.is_empty() {
                return Err(YamlError::EmptyFile);
            }
            let doc = docs[0].clone();
            let document = Arc::new(RwLock::new(doc));

            documents.push(document);
        }

        if validate {
            Order::parse_all_from_yaml(documents.clone(), None)?;
            Scenario::parse_all_from_yaml(documents.clone(), None)?;
            Deployment::parse_all_from_yaml(documents.clone(), None)?;
        }

        Ok(DotrainYaml { documents })
    }

    fn from_documents(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Self {
        DotrainYaml { documents }
    }
}

impl DotrainYaml {
    pub fn get_order_keys(&self) -> Result<Vec<String>, YamlError> {
        let orders = Order::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(orders.keys().cloned().collect())
    }
    pub fn get_order(&self, key: &str) -> Result<Order, YamlError> {
        Order::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_scenario_keys(&self) -> Result<Vec<String>, YamlError> {
        let scenarios = Scenario::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(scenarios.keys().cloned().collect())
    }
    pub fn get_scenario(&self, key: &str) -> Result<Scenario, YamlError> {
        Scenario::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_deployment_keys(&self) -> Result<Vec<String>, YamlError> {
        let deployments = Deployment::parse_all_from_yaml(self.documents.clone(), None)?;
        Ok(deployments.keys().cloned().collect())
    }
    pub fn get_deployment(&self, key: &str) -> Result<Deployment, YamlError> {
        Deployment::parse_from_yaml(self.documents.clone(), key, None)
    }

    pub fn get_gui(&self) -> Result<Option<Gui>, YamlError> {
        Gui::parse_from_yaml_optional(self.documents.clone(), None)
    }
}

impl Serialize for DotrainYaml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.documents.len()))?;
        for doc in &self.documents {
            let yaml_str = Self::get_yaml_string(doc.clone()).map_err(serde::ser::Error::custom)?;
            seq.serialize_element(&yaml_str)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for DotrainYaml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DotrainYamlVisitor;

        impl<'de> Visitor<'de> for DotrainYamlVisitor {
            type Value = DotrainYaml;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of YAML documents as strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut documents = Vec::new();

                while let Some(doc_str) = seq.next_element::<String>()? {
                    let docs =
                        StrictYamlLoader::load_from_str(&doc_str).map_err(de::Error::custom)?;
                    if docs.is_empty() {
                        return Err(de::Error::custom("Empty YAML document"));
                    }
                    let doc = docs[0].clone();
                    documents.push(Arc::new(RwLock::new(doc)));
                }

                Ok(DotrainYaml { documents })
            }
        }

        deserializer.deserialize_seq(DotrainYamlVisitor)
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
    deployments:
        deployment1:
            order: order1
            scenario: scenario1.scenario2
        deployment2:
            order: order1
            scenario: scenario1
    gui:
        name: Test gui
        description: Test description
        deployments:
            deployment1:
                name: Test deployment
                description: Test description
                deposits:
                    - token: token1
                      presets:
                        - 100
                        - 2000
                fields:
                    - binding: key1
                      name: Binding test
                      presets:
                        - value: value2
                select-tokens:
                    - token2
    "#;

    const HANDLEBARS_YAML: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
    tokens:
        token1:
            network: mainnet
            address: 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
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
                key1: ${order.inputs.0.token.address}
            deployer: deployer1
            scenarios:
                scenario2:
                    bindings:
                        key2: ${order.outputs.0.token.address}
    deployments:
        deployment1:
            order: order1
            scenario: scenario1.scenario2
    gui:
        name: Test gui
        description: Test description
        deployments:
            deployment1:
                name: Test deployment
                description: Test description
                deposits:
                    - token: token1
                      presets:
                        - 100
                        - 2000
                fields:
                    - binding: key1
                      name: Binding for ${order.inputs.0.token.label}
                      description: With token symbol ${order.inputs.0.token.symbol}
                      presets:
                        - value: value2
    "#;

    #[test]
    fn test_full_yaml() {
        let ob_yaml = OrderbookYaml::new(vec![FULL_YAML.to_string()], false).unwrap();
        let dotrain_yaml = DotrainYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

        assert_eq!(dotrain_yaml.get_order_keys().unwrap().len(), 1);
        let order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        let input = order.inputs.first().unwrap();
        assert_eq!(
            *input.token.clone().as_ref().unwrap(),
            ob_yaml.get_token("token1").unwrap().into()
        );
        assert_eq!(input.vault_id, Some(U256::from(1)));
        let output = order.outputs.first().unwrap();
        assert_eq!(
            *output.token.as_ref().unwrap(),
            ob_yaml.get_token("token2").unwrap().into()
        );
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
        let scenario2 = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();
        assert_eq!(scenario2.bindings.len(), 2);
        assert_eq!(scenario2.bindings.get("key1").unwrap(), "value1");
        assert_eq!(scenario2.bindings.get("key2").unwrap(), "value2");
        assert_eq!(
            *scenario2.deployer.as_ref(),
            ob_yaml.get_deployer("deployer1").unwrap()
        );

        let deployment_keys = dotrain_yaml.get_deployment_keys().unwrap();
        assert_eq!(deployment_keys.len(), 2);
        let deployment = dotrain_yaml.get_deployment("deployment1").unwrap();
        assert_eq!(
            deployment.order,
            dotrain_yaml.get_order("order1").unwrap().into()
        );
        assert_eq!(
            deployment.scenario,
            dotrain_yaml
                .get_scenario("scenario1.scenario2")
                .unwrap()
                .into()
        );
        let deployment = dotrain_yaml.get_deployment("deployment2").unwrap();
        assert_eq!(
            deployment.order,
            dotrain_yaml.get_order("order1").unwrap().into()
        );
        assert_eq!(
            deployment.scenario,
            dotrain_yaml.get_scenario("scenario1").unwrap().into()
        );
        assert_eq!(
            Deployment::parse_order_key(dotrain_yaml.documents.clone(), "deployment1").unwrap(),
            "order1"
        );
        assert_eq!(
            Deployment::parse_order_key(dotrain_yaml.documents.clone(), "deployment2").unwrap(),
            "order1"
        );

        let gui = dotrain_yaml.get_gui().unwrap().unwrap();
        assert_eq!(gui.name, "Test gui");
        assert_eq!(gui.description, "Test description");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = gui.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.name, "Test deployment");
        assert_eq!(deployment.description, "Test description");
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = &deployment.deposits[0];
        assert_eq!(
            *deposit.token.as_ref().unwrap(),
            ob_yaml.get_token("token1").unwrap().into()
        );
        assert_eq!(deposit.presets.len(), 2);
        assert_eq!(deposit.presets[0], "100".to_string());
        assert_eq!(deposit.presets[1], "2000".to_string());
        assert_eq!(deployment.fields.len(), 1);
        let field = &deployment.fields[0];
        assert_eq!(field.binding, "key1");
        assert_eq!(field.name, "Binding test");
        let presets = field.presets.as_ref().unwrap();
        assert_eq!(presets[0].value, "value2");
        let select_tokens = deployment.select_tokens.as_ref().unwrap();
        assert_eq!(select_tokens.len(), 1);
        assert_eq!(select_tokens[0], "token2");

        let deployment_keys = Gui::parse_deployment_keys(dotrain_yaml.documents.clone()).unwrap();
        assert_eq!(deployment_keys.len(), 1);
        assert_eq!(deployment_keys[0], "deployment1");

        let select_tokens =
            Gui::parse_select_tokens(dotrain_yaml.documents.clone(), "deployment1").unwrap();
        assert!(select_tokens.is_some());
        assert_eq!(select_tokens.unwrap()[0], "token2");

        let select_tokens =
            Gui::parse_select_tokens(dotrain_yaml.documents.clone(), "deployment2").unwrap();
        assert!(select_tokens.is_none());
    }

    #[test]
    fn test_update_vault_ids() {
        let yaml = r#"
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
        orders:
            order1:
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        "#;
        let dotrain_yaml = DotrainYaml::new(vec![yaml.to_string()], false).unwrap();

        let mut order = dotrain_yaml.get_order("order1").unwrap();

        assert!(order.inputs[0].vault_id.is_none());
        assert!(order.outputs[0].vault_id.is_none());

        let updated_order = order.populate_vault_ids().unwrap();

        // After population, all vault IDs should be set and equal
        assert!(updated_order.inputs[0].vault_id.is_some());
        assert!(updated_order.outputs[0].vault_id.is_some());
        assert_eq!(
            updated_order.inputs[0].vault_id,
            updated_order.outputs[0].vault_id
        );

        let order_after = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(
            order_after.inputs[0].vault_id,
            updated_order.inputs[0].vault_id
        );
        assert_eq!(
            order_after.outputs[0].vault_id,
            updated_order.outputs[0].vault_id
        );

        // Populate vault IDs should not change if the vault IDs are already set
        let dotrain_yaml = DotrainYaml::new(vec![FULL_YAML.to_string()], false).unwrap();
        let mut order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(2)));
        order.populate_vault_ids().unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(2)));
    }

    #[test]
    fn test_update_vault_id() {
        let yaml = r#"
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
        orders:
            order1:
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        "#;
        let dotrain_yaml = DotrainYaml::new(vec![yaml.to_string()], false).unwrap();
        let mut order = dotrain_yaml.get_order("order1").unwrap();

        assert!(order.inputs[0].vault_id.is_none());
        assert!(order.outputs[0].vault_id.is_none());

        let mut updated_order = order.update_vault_id(true, 0, "1".to_string()).unwrap();
        let updated_order = updated_order
            .update_vault_id(false, 0, "11".to_string())
            .unwrap();

        assert_eq!(updated_order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(updated_order.outputs[0].vault_id, Some(U256::from(11)));

        let mut order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(11)));

        let mut updated_order = order.update_vault_id(true, 0, "3".to_string()).unwrap();
        let updated_order = updated_order
            .update_vault_id(false, 0, "33".to_string())
            .unwrap();
        assert_eq!(updated_order.inputs[0].vault_id, Some(U256::from(3)));
        assert_eq!(updated_order.outputs[0].vault_id, Some(U256::from(33)));

        let order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(3)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(33)));
    }

    #[test]
    fn test_update_bindings() {
        // Parent scenario
        {
            let dotrain_yaml = DotrainYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

            let mut scenario = dotrain_yaml.get_scenario("scenario1").unwrap();

            assert_eq!(scenario.bindings.len(), 1);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");

            let updated_scenario = scenario
                .update_bindings(HashMap::from([("key1".to_string(), "value2".to_string())]))
                .unwrap();

            assert_eq!(updated_scenario.bindings.len(), 1);
            assert_eq!(updated_scenario.bindings.get("key1").unwrap(), "value2");

            let scenario = dotrain_yaml.get_scenario("scenario1").unwrap();
            assert_eq!(scenario.bindings.len(), 1);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value2");
        }

        // Child scenario
        {
            let dotrain_yaml = DotrainYaml::new(vec![FULL_YAML.to_string()], false).unwrap();

            let mut scenario = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();

            assert_eq!(scenario.bindings.len(), 2);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");
            assert_eq!(scenario.bindings.get("key2").unwrap(), "value2");

            let updated_scenario = scenario
                .update_bindings(HashMap::from([
                    ("key1".to_string(), "value3".to_string()),
                    ("key2".to_string(), "value4".to_string()),
                ]))
                .unwrap();

            assert_eq!(updated_scenario.bindings.len(), 2);
            assert_eq!(updated_scenario.bindings.get("key1").unwrap(), "value3");
            assert_eq!(updated_scenario.bindings.get("key2").unwrap(), "value4");

            let scenario = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();
            assert_eq!(scenario.bindings.len(), 2);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value3");
            assert_eq!(scenario.bindings.get("key2").unwrap(), "value4");
        }
    }

    #[test]
    fn test_handlebars() {
        let dotrain_yaml = DotrainYaml::new(vec![HANDLEBARS_YAML.to_string()], false).unwrap();

        let gui = dotrain_yaml.get_gui().unwrap().unwrap();
        let deployment = gui.deployments.get("deployment1").unwrap();

        assert_eq!(
            deployment.deployment.scenario.bindings.get("key1").unwrap(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(
            deployment.deployment.scenario.bindings.get("key2").unwrap(),
            "0x0000000000000000000000000000000000000002"
        );

        assert_eq!(deployment.fields[0].name, "Binding for Wrapped Ether");
        assert_eq!(
            deployment.fields[0].description,
            Some("With token symbol WETH".to_string())
        );
    }

    #[test]
    fn test_parse_orders_missing_token() {
        let yaml_prefix = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
deployers:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
scenarios:
    scenario1:
        deployer: mainnet
        bindings:
            key1: value1
deployments:
    deployment1:
        order: order1
        scenario: scenario1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token-one
                  presets:
                    - 1
                - token: token-two
                  presets:
                    - 1
                - token: token-three
                  presets:
                    - 1
            fields:
                - binding: key1
                  name: test
                  presets:
                    - value: 1
            select-tokens:
                - token-one
                - token-two
"#;
        let missing_input_token_yaml = format!(
            "{yaml_prefix}
orders:
    order1:
        inputs:
            - token: token-three
        outputs:
            - token: token-two
            - token: token-three
        "
        );
        let missing_output_token_yaml = format!(
            "{yaml_prefix}
orders:
    order1:
        inputs:
            - token: token-one
            - token: token-two
        outputs:
            - token: token-three
        "
        );

        let dotrain_yaml = DotrainYaml::new(vec![missing_input_token_yaml], false).unwrap();
        let error = dotrain_yaml.get_gui().unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "yaml data for token: token-three not found in input index: 0 in order: order1"
                    .to_string()
            )
        );

        let dotrain_yaml = DotrainYaml::new(vec![missing_output_token_yaml], false).unwrap();
        let error = dotrain_yaml.get_gui().unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "yaml data for token: token-three not found in output index: 0 in order: order1"
                    .to_string()
            )
        );
    }
}
