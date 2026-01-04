use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg,
    scenario::ScenarioCfg,
    yaml::{
        dotrain::{DotrainYaml, DotrainYamlValidation},
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlParsable,
    },
};
use std::collections::HashMap;

fn get_dotrain_yaml(dotrain: String, settings: Option<String>) -> CommandResult<DotrainYaml> {
    let frontmatter = RainDocument::get_front_matter(&dotrain)
        .unwrap_or("")
        .to_string();
    let sources = if let Some(settings) = settings {
        vec![frontmatter, settings]
    } else {
        vec![frontmatter]
    };
    Ok(DotrainYaml::new(sources, DotrainYamlValidation::default())?)
}

#[tauri::command]
pub async fn check_settings_errors(text: Vec<String>) -> CommandResult<()> {
    OrderbookYaml::new(
        text.clone(),
        OrderbookYamlValidation {
            networks: true,
            remote_networks: false,
            tokens: false,
            remote_tokens: false,
            subgraphs: true,
            orderbooks: true,
            metaboards: true,
            deployers: true,
            local_db_remotes: true,
            local_db_sync: true
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn check_dotrain_with_settings_errors(
    dotrain: String,
    settings: Vec<String>,
) -> CommandResult<()> {
    OrderbookYaml::new(
        settings.clone(),
        OrderbookYamlValidation {
            networks: true,
            remote_networks: false,
            tokens: false,
            remote_tokens: false,
            subgraphs: true,
            orderbooks: true,
            metaboards: true,
            deployers: true,
            local_db_remotes: true,
            local_db_sync: true
        },
    )?;
    let mut sources = vec![RainDocument::get_front_matter(&dotrain)
        .unwrap_or("")
        .to_string()];
    sources.extend(settings);

    DotrainYaml::new(
        sources,
        DotrainYamlValidation {
            deployments: true,
            scenarios: true,
            orders: true,
        },
    )?;

    Ok(())
}

#[tauri::command]
pub async fn get_deployments(
    dotrain: String,
    settings: Option<String>,
) -> CommandResult<HashMap<String, DeploymentCfg>> {
    Ok(get_dotrain_yaml(dotrain, settings)?.get_deployments()?)
}

#[tauri::command]
pub async fn get_scenarios(
    dotrain: String,
    settings: Option<String>,
) -> CommandResult<HashMap<String, ScenarioCfg>> {
    Ok(get_dotrain_yaml(dotrain, settings)?.get_scenarios()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOTRAIN: &str = r#"
version: 4
networks:
    some-network:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    test: https://metaboard.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        deployment-block: 12345

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2

scenarios:
    some-scenario:
        deployer: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300

orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      deployer: some-deployer
      orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: some-scenario.sub-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#;

    #[tokio::test]
    async fn test_get_deployments() {
        let deployments = get_deployments(DOTRAIN.to_string(), None).await.unwrap();
        assert_eq!(deployments.len(), 2);
        assert!(deployments.contains_key("some-deployment"));
        assert!(deployments.contains_key("other-deployment"));
    }

    #[tokio::test]
    async fn test_get_scenarios() {
        let scenarios = get_scenarios(DOTRAIN.to_string(), None).await.unwrap();
        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.contains_key("some-scenario"));
        assert!(scenarios.contains_key("some-scenario.sub-scenario"));
    }
}
