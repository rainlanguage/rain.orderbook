use rain_orderbook_app_settings::{config::Config, ParseConfigError};
use wasm_bindgen_utils::prelude::*;

#[wasm_bindgen(js_name = "parseSettings", unchecked_return_type = "Config")]
pub async fn parse_settings(
    settings: Vec<String>,
    validate: Option<bool>,
) -> Result<JsValue, ParseConfigError> {
    let config = Config::try_from_settings(settings, validate.unwrap_or(false))?;
    Ok(to_js_value(&config)?)
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    const ORDERBOOK_YAML: &str = r#"
    raindex-version: 0.1.0
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpc: https://testnet.infura.io
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
    async fn test_parse_settings() {
        let config = Config::try_from_settings(
            vec![ORDERBOOK_YAML.to_string(), DOTRAIN_YAML.to_string()],
            false,
        )
        .unwrap();
        let js_value = parse_settings(
            vec![ORDERBOOK_YAML.to_string(), DOTRAIN_YAML.to_string()],
            None,
        )
        .await
        .unwrap();
        assert_eq!(config, Config::try_from_js_value(js_value).unwrap());
    }
}
