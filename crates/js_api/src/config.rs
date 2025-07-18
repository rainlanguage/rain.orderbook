use rain_orderbook_app_settings::new_config::{NewConfig, ParseConfigError};
use wasm_bindgen_utils::prelude::*;

/// Parses YAML configuration files and creates a unified configuration object.
///
/// Combines multiple YAML files into a single configuration, with validation support.
///
/// ## Examples
///
/// ```javascript
/// const result = parseYaml([orderbookYaml, dotrainYaml], true);
/// if (result.error) {
///   console.error("Parse failed:", result.error.readableMsg);
///   return;
/// }
/// const config = result.value;
/// // Do something with the config
/// ```
#[wasm_export(
    js_name = "parseYaml",
    unchecked_return_type = "NewConfig",
    return_description = "Unified configuration object from parsed YAML files"
)]
pub fn parse_yaml(
    #[wasm_export(param_description = "Array of YAML configuration strings to parse")]
    yaml_list: Vec<String>,
    #[wasm_export(
        param_description = "Whether to perform strict validation (optional, defaults to false)"
    )]
    validate: Option<bool>,
) -> Result<NewConfig, ParseConfigError> {
    Ok(NewConfig::try_from_yaml(
        yaml_list,
        validate.unwrap_or(false),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    const ORDERBOOK_YAML: &str = r#"
    version: 2
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpcs:
                - https://testnet.infura.io
            chain-id: 1337
    subgraphs:
        mainnet: https://mainnet-subgraph.com
        testnet: https://testnet-subgraph.com
    metaboards:
        mainnet: https://mainnet-metaboard.com
        testnet: https://testnet-metaboard.com
    orderbooks:
        mainnet:
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            network: mainnet
            subgraph: mainnet
        testnet:
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            network: testnet
            subgraph: testnet
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
        scenario1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
        deployer2:
            address: 0x0000000000000000000000000000000000000003
            network: testnet
    sentry: true
    "#;
    const DOTRAIN_YAML: &str = r#"
    orders:
        order1:
            deployer: scenario1
            orderbook: mainnet
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
            scenarios:
                scenario2:
                    bindings:
                        key2: value2
                    runs: 10
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
        short-description: Test short description
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
                    - key: token2
                      name: Test token
                      description: Test description
    charts:
        chart1:
            scenario: scenario1.scenario2
            plots:
                plot1:
                    title: Test title
                    subtitle: Test subtitle
                    marks:
                        - type: dot
                          options:
                            x: 1
                            y: 2
                            r: 3
                            fill: red
                            stroke: blue
                            transform:
                                type: hexbin
                                content:
                                    outputs:
                                        x: 1
                                        y: 2
                                        r: 3
                                        z: 4
                                        stroke: green
                                        fill: blue
                                    options:
                                        x: 1
                                        y: 2
                                        bin-width: 10
                        - type: line
                          options:
                            transform:
                                type: binx
                                content:
                                    outputs:
                                        x: 1
                                    options:
                                        thresholds: 10
                        - type: recty
                          options:
                            x0: 1
                            x1: 2
                            y0: 3
                            y1: 4
                    x:
                       label: Test x label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    y:
                       label: Test y label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    margin: 10
                    margin-left: 20
                    margin-right: 30
                    margin-top: 40
                    margin-bottom: 50
                    inset: 60
    "#;

    #[wasm_bindgen_test]
    fn test_parse_yaml() {
        let config = NewConfig::try_from_yaml(
            vec![ORDERBOOK_YAML.to_string(), DOTRAIN_YAML.to_string()],
            false,
        )
        .unwrap();
        let parsed_config = parse_yaml(
            vec![ORDERBOOK_YAML.to_string(), DOTRAIN_YAML.to_string()],
            None,
        )
        .unwrap();
        assert_eq!(config, parsed_config);
    }
}
