use super::orders::{GetOrdersFilters, GetOrdersTokenFilter};
use super::*;
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_order_candidates_for_pair, build_take_orders_config_from_sell_simulation,
    simulate_sell_over_candidates, MinReceiveMode, SimulatedSellResult, TakeOrderCandidate,
};
use alloy::primitives::{Address, Bytes};
use alloy::sol_types::SolCall;
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::takeOrders3Call;
use std::collections::HashMap;
use std::ops::Div;
use std::str::FromStr;

/// Combined result for generating takeOrders3 calldata and price info.
///
/// `calldata` can be sent directly as transaction data; `effective_price` and
/// `prices` provide blended and per-leg prices (sell per 1 buy).
#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersCalldataResult {
    /// The orderbook contract address to call with this calldata
    #[tsify(type = "Address")]
    pub orderbook: Address,
    /// ABI-encoded calldata for `IOrderBookV5.takeOrders3` (hex in JS)
    #[tsify(type = "Hex")]
    pub calldata: Bytes,
    /// Blended sell per 1 buy (totalSell / totalBuy)
    #[tsify(type = "Hex")]
    pub effective_price: Float,
    /// Per-leg prices (sell per 1 buy), sorted best (cheapest) to worst
    #[tsify(type = "Hex[]")]
    pub prices: Vec<Float>,
}
impl_wasm_traits!(TakeOrdersCalldataResult);

#[wasm_export]
impl RaindexClient {
    /// Generates ABI-encoded calldata for `takeOrders3()` using an exact-in builder.
    ///
    /// Discovers orders for the given `(chainId, sellToken, buyToken)` pair, simulates
    /// how much `buyToken` is received for the exact `sellAmount` budget, applies the
    /// `minReceiveMode` policy, and returns both calldata and price information.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getTakeOrdersCalldata(
    ///   137,                 // chainId
    ///   "0xSELL...",         // sellToken
    ///   "0xBUY...",          // buyToken
    ///   sellAmountFloat,     // exact sell amount as Float hex string
    ///   "partial",           // or "exact"
    /// );
    /// if (result.error) {
    ///   console.error("Failed to build takeOrders calldata:", result.error.readableMsg);
    ///   return;
    /// }
    /// const { calldata, effectivePrice, prices } = result.value;
    /// // Use calldata for transaction, show prices in UI, etc.
    /// ```
    #[wasm_export(
        js_name = "getTakeOrdersCalldata",
        return_description = "Encoded takeOrders3 calldata and price information",
        unchecked_return_type = "TakeOrdersCalldataResult"
    )]
    pub async fn get_take_orders_calldata(
        &self,
        #[wasm_export(
            js_name = "chainId",
            param_description = "Chain ID of the target network"
        )]
        chain_id: u32,
        #[wasm_export(
            js_name = "sellToken",
            param_description = "Token address the taker will GIVE",
            unchecked_param_type = "Address"
        )]
        sell_token: String,
        #[wasm_export(
            js_name = "buyToken",
            param_description = "Token address the taker will RECEIVE",
            unchecked_param_type = "Address"
        )]
        buy_token: String,
        #[wasm_export(
            js_name = "sellAmount",
            param_description = "Exact sell amount as a Float hex string in sellToken units",
            unchecked_param_type = "Hex"
        )]
        sell_amount: String,
        #[wasm_export(
            js_name = "minReceiveMode",
            param_description = "Minimum receive policy: partial or exact"
        )]
        min_receive_mode: MinReceiveMode,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let sell_token_addr = Address::from_str(&sell_token)?;
        let buy_token_addr = Address::from_str(&buy_token)?;
        let sell_amount_float = Float::parse(sell_amount)?;

        let filters = GetOrdersFilters {
            owners: vec![],
            active: Some(true),
            order_hash: None,
            tokens: Some(GetOrdersTokenFilter {
                inputs: Some(vec![sell_token_addr]),
                outputs: Some(vec![buy_token_addr]),
            }),
        };

        let orders = self
            .get_orders(Some(ChainIds(vec![chain_id])), Some(filters), None)
            .await?;

        if orders.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let rpc_urls = self.get_rpc_urls_for_chain(chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls)?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates = build_take_order_candidates_for_pair(
            &orders,
            sell_token_addr,
            buy_token_addr,
            Some(block_number),
            None,
        )
        .await?;

        if candidates.is_empty() {
            return Err(RaindexError::NoLiquidity);
        }

        let (best_orderbook, best_sim) =
            select_best_orderbook_simulation(candidates, sell_amount_float)?;

        let built = build_take_orders_config_from_sell_simulation(best_sim, min_receive_mode)?
            .ok_or(RaindexError::NoLiquidity)?;

        let calldata_bytes = takeOrders3Call {
            config: built.config,
        }
        .abi_encode();
        let calldata = Bytes::copy_from_slice(&calldata_bytes);

        let zero = Float::zero()?;
        let effective_price = if built.sim.total_buy_amount.gt(zero)? {
            built
                .sim
                .total_sell_amount
                .div(built.sim.total_buy_amount)?
        } else {
            zero
        };

        let prices: Vec<Float> = built
            .sim
            .legs
            .iter()
            .map(|leg| leg.candidate.ratio)
            .collect();

        Ok(TakeOrdersCalldataResult {
            orderbook: best_orderbook,
            calldata,
            effective_price,
            prices,
        })
    }
}

fn select_best_orderbook_simulation(
    candidates: Vec<TakeOrderCandidate>,
    sell_budget: Float,
) -> Result<(Address, SimulatedSellResult), RaindexError> {
    let mut orderbook_candidates: HashMap<Address, Vec<TakeOrderCandidate>> = HashMap::new();
    for candidate in candidates {
        orderbook_candidates
            .entry(candidate.orderbook)
            .or_default()
            .push(candidate);
    }

    let mut best_result: Option<(Address, SimulatedSellResult)> = None;

    for (orderbook, candidates) in orderbook_candidates {
        let sim = simulate_sell_over_candidates(candidates, sell_budget)?;

        if sim.legs.is_empty() {
            continue;
        }

        let is_better = match &best_result {
            None => true,
            Some((_, best_sim)) => sim.total_buy_amount.gt(best_sim.total_buy_amount)?,
        };

        if is_better {
            best_result = Some((orderbook, sim));
        }
    }

    best_result.ok_or(RaindexError::NoLiquidity)
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::take_orders::MinReceiveMode;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::{from_js_value, to_js_value};

        #[wasm_bindgen_test]
        fn test_min_receive_mode_serialization() {
            let partial = MinReceiveMode::Partial;
            let exact = MinReceiveMode::Exact;

            let partial_js = to_js_value(&partial).unwrap();
            let exact_js = to_js_value(&exact).unwrap();

            let partial_back: MinReceiveMode = from_js_value(partial_js).unwrap();
            let exact_back: MinReceiveMode = from_js_value(exact_js).unwrap();

            assert!(matches!(partial_back, MinReceiveMode::Partial));
            assert!(matches!(exact_back, MinReceiveMode::Exact));
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::super::select_best_orderbook_simulation;
        use crate::add_order::AddOrderArgs;
        use crate::dotrain_order::DotrainOrder;
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::RaindexClient;
        use crate::raindex_client::RaindexError;
        use crate::take_orders::{MinReceiveMode, TakeOrderCandidate};
        use alloy::hex::encode_prefixed;
        use alloy::primitives::{Address, B256, U256};
        use alloy::sol_types::{SolCall, SolValue};
        use httpmock::MockServer;
        use rain_math_float::Float;
        use rain_orderbook_app_settings::spec_version::SpecVersion;
        use rain_orderbook_bindings::IOrderBookV5::{takeOrders3Call, EvaluableV4, OrderV4, IOV2};
        use rain_orderbook_subgraph_client::types::common::{
            SgBigInt, SgBytes, SgErc20, SgOrderbook, SgVault,
        };
        use rain_orderbook_test_fixtures::LocalEvm;
        use serde_json::json;

        fn make_basic_order(input_token: Address, output_token: Address) -> OrderV4 {
            OrderV4 {
                owner: Address::from([1u8; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([2u8; 20]),
                    store: Address::from([3u8; 20]),
                    bytecode: alloy::primitives::Bytes::from(vec![0x01, 0x02]),
                },
                validInputs: vec![IOV2 {
                    token: input_token,
                    vaultId: U256::from(100).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: output_token,
                    vaultId: U256::from(200).into(),
                }],
            }
        }

        fn make_candidate(
            orderbook: Address,
            max_output: Float,
            ratio: Float,
        ) -> TakeOrderCandidate {
            TakeOrderCandidate {
                orderbook,
                order: make_basic_order(Address::from([4u8; 20]), Address::from([5u8; 20])),
                input_io_index: 0,
                output_io_index: 0,
                max_output,
                ratio,
            }
        }

        #[test]
        fn test_select_best_orderbook_single_orderbook() {
            let ob1 = Address::from([0x11u8; 20]);
            let max_output = Float::parse("10".to_string()).unwrap();
            let ratio = Float::parse("2".to_string()).unwrap();
            let candidate = make_candidate(ob1, max_output, ratio);
            let candidates = vec![candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (addr, sim) = result.unwrap();
            assert_eq!(addr, ob1);
            assert!(!sim.legs.is_empty());
            assert!(sim.total_buy_amount.gt(Float::zero().unwrap()).unwrap());
            assert!(sim.total_sell_amount.gt(Float::zero().unwrap()).unwrap());
        }

        #[test]
        fn test_select_best_orderbook_multiple_books_picks_best() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("5".to_string()).unwrap();
            let ob1_ratio = Float::parse("1".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("8".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::parse("100".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();
            assert_eq!(winner, ob2);
            let expected_buy = Float::parse("8".to_string()).unwrap();
            assert!(sim.total_buy_amount.eq(expected_buy).unwrap());
        }

        #[test]
        fn test_select_best_orderbook_skips_empty_sims() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("10".to_string()).unwrap();
            let ob1_ratio = Float::parse("2".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("5".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::zero().unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(matches!(result, Err(RaindexError::NoLiquidity)));
        }

        #[test]
        fn test_select_best_orderbook_all_empty_returns_no_liquidity() {
            let ob1 = Address::from([0x11u8; 20]);
            let ob2 = Address::from([0x22u8; 20]);

            let ob1_max_output = Float::parse("10".to_string()).unwrap();
            let ob1_ratio = Float::parse("2".to_string()).unwrap();
            let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

            let ob2_max_output = Float::parse("5".to_string()).unwrap();
            let ob2_ratio = Float::parse("1".to_string()).unwrap();
            let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

            let candidates = vec![ob1_candidate, ob2_candidate];
            let sell_budget = Float::zero().unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_err());
            assert!(matches!(result, Err(RaindexError::NoLiquidity)));
        }

        #[test]
        fn test_select_best_orderbook_skips_empty_picks_valid() {
            let ob_empty = Address::from([0x11u8; 20]);
            let ob_valid = Address::from([0x22u8; 20]);

            let empty_max_output = Float::parse("10".to_string()).unwrap();
            let empty_ratio = Float::parse("1000000".to_string()).unwrap();
            let empty_candidate = make_candidate(ob_empty, empty_max_output, empty_ratio);

            let valid_max_output = Float::parse("5".to_string()).unwrap();
            let valid_ratio = Float::parse("1".to_string()).unwrap();
            let valid_candidate = make_candidate(ob_valid, valid_max_output, valid_ratio);

            let candidates = vec![empty_candidate, valid_candidate];
            let sell_budget = Float::parse("10".to_string()).unwrap();

            let result = select_best_orderbook_simulation(candidates, sell_budget);

            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();
            assert_eq!(winner, ob_valid);
            assert!(!sim.legs.is_empty());
            assert!(sim.total_buy_amount.gt(Float::zero().unwrap()).unwrap());
        }

        fn get_minimal_yaml_for_chain(
            chain_id: u32,
            rpc_url: &str,
            sg_url: &str,
            orderbook_address: &str,
        ) -> String {
            format!(
                r#"
version: {spec_version}
networks:
    test-network:
        rpcs:
            - {rpc_url}
        chain-id: {chain_id}
        network-id: {chain_id}
        currency: ETH
subgraphs:
    test-sg: {sg_url}
metaboards:
    test-mb: http://localhost:0/notused
orderbooks:
    test-orderbook:
        address: {orderbook_address}
        network: test-network
        subgraph: test-sg
        local-db-remote: remote
        deployment-block: 0
deployers:
    test-deployer:
        network: test-network
        address: 0x1111111111111111111111111111111111111111
tokens:
    test-token:
        network: test-network
        address: 0x2222222222222222222222222222222222222222
        decimals: 18
        label: TestToken
        symbol: TST
"#,
                spec_version = SpecVersion::current(),
                chain_id = chain_id,
                rpc_url = rpc_url,
                sg_url = sg_url,
                orderbook_address = orderbook_address,
            )
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_no_orders_returns_no_liquidity() {
            let sg_server = MockServer::start_async().await;

            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": []
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": []
                    }
                }));
            });

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    "http://localhost:0/unused_rpc1",
                    "http://localhost:0/unused_rpc2",
                )],
                None,
            )
            .unwrap();

            let res = client
                .get_take_orders_calldata(
                    1,
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                    "1".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when subgraph returns empty orders, got: {:?}",
                res
            );
        }

        struct TestSetup {
            local_evm: LocalEvm,
            owner: Address,
            token1: Address,
            token2: Address,
            token1_sg: SgErc20,
            token2_sg: SgErc20,
            orderbook: Address,
        }

        async fn setup_local_evm_test() -> TestSetup {
            let mut local_evm = LocalEvm::new().await;
            let owner = local_evm.signer_wallets[0].default_signer().address();

            let token1 = local_evm
                .deploy_new_token("Token1", "Token1", 18, U256::MAX, owner)
                .await;
            let token2 = local_evm
                .deploy_new_token("Token2", "Token2", 18, U256::MAX, owner)
                .await;
            let orderbook = *local_evm.orderbook.address();

            TestSetup {
                token1: *token1.address(),
                token2: *token2.address(),
                token1_sg: SgErc20 {
                    id: SgBytes(token1.address().to_string()),
                    address: SgBytes(token1.address().to_string()),
                    name: Some("Token1".to_string()),
                    symbol: Some("Token1".to_string()),
                    decimals: Some(SgBigInt(18.to_string())),
                },
                token2_sg: SgErc20 {
                    id: SgBytes(token2.address().to_string()),
                    address: SgBytes(token2.address().to_string()),
                    name: Some("Token2".to_string()),
                    symbol: Some("Token2".to_string()),
                    decimals: Some(SgBigInt(18.to_string())),
                },
                local_evm,
                owner,
                orderbook,
            }
        }

        fn create_dotrain_config(setup: &TestSetup, max_output: &str, ratio: &str) -> String {
            format!(
                r#"
version: {spec_version}
networks:
    test-network:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
deployers:
    test-deployer:
        network: test-network
        address: {deployer}
tokens:
    t1:
        network: test-network
        address: {token1}
        decimals: 18
        label: Token1
        symbol: Token1
    t2:
        network: test-network
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
orderbook:
    test-orderbook:
        address: {orderbook}
orders:
    test-order:
        inputs:
            - token: t1
            - token: t2
        outputs:
            - token: t1
              vault-id: 0x01
            - token: t2
              vault-id: 0x01
scenarios:
    test-scenario:
        deployer: test-deployer
        bindings:
            max-amount: 1000
deployments:
    test-deployment:
        scenario: test-scenario
        order: test-order
---
#max-amount !Max output amount
#calculate-io
amount price: {max_output} {ratio};
#handle-add-order
:;
#handle-io
:;
"#,
                rpc_url = setup.local_evm.url(),
                orderbook = setup.orderbook,
                deployer = setup.local_evm.deployer.address(),
                token1 = setup.token1,
                token2 = setup.token2,
                spec_version = SpecVersion::current(),
                max_output = max_output,
                ratio = ratio,
            )
        }

        async fn deploy_order(setup: &TestSetup, dotrain: String) -> (String, B256) {
            let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
            let deployment = dotrain_order
                .dotrain_yaml()
                .get_deployment("test-deployment")
                .unwrap();
            let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment)
                .await
                .unwrap()
                .try_into_call(vec![setup.local_evm.url()])
                .await
                .unwrap()
                .abi_encode();

            let (event, _) = setup.local_evm.add_order(&calldata, setup.owner).await;
            let order_bytes = encode_prefixed(event.order.abi_encode());
            let order_hash = B256::from(event.orderHash);
            (order_bytes, order_hash)
        }

        fn create_vault(vault_id: B256, setup: &TestSetup, token: &SgErc20) -> SgVault {
            SgVault {
                id: SgBytes(vault_id.to_string()),
                token: token.clone(),
                balance: SgBytes(Float::parse("6".to_string()).unwrap().as_hex()),
                vault_id: SgBytes(vault_id.to_string()),
                owner: SgBytes(setup.local_evm.anvil.addresses()[0].to_string()),
                orderbook: SgOrderbook {
                    id: SgBytes(setup.orderbook.to_string()),
                },
                orders_as_input: vec![],
                orders_as_output: vec![],
                balance_changes: vec![],
            }
        }

        fn create_sg_order_json(
            setup: &TestSetup,
            order_bytes: &str,
            order_hash: B256,
            inputs: Vec<SgVault>,
            outputs: Vec<SgVault>,
        ) -> serde_json::Value {
            let inputs_json: Vec<serde_json::Value> = inputs
                .iter()
                .map(|v| {
                    json!({
                        "id": v.id.0,
                        "owner": v.owner.0,
                        "vaultId": v.vault_id.0,
                        "balance": v.balance.0,
                        "token": {
                            "id": v.token.id.0,
                            "address": v.token.address.0,
                            "name": v.token.name.clone().unwrap_or_default(),
                            "symbol": v.token.symbol.clone().unwrap_or_default(),
                            "decimals": v.token.decimals.clone().map(|d| d.0).unwrap_or_default()
                        },
                        "orderbook": { "id": v.orderbook.id.0 },
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    })
                })
                .collect();

            let outputs_json: Vec<serde_json::Value> = outputs
                .iter()
                .map(|v| {
                    json!({
                        "id": v.id.0,
                        "owner": v.owner.0,
                        "vaultId": v.vault_id.0,
                        "balance": v.balance.0,
                        "token": {
                            "id": v.token.id.0,
                            "address": v.token.address.0,
                            "name": v.token.name.clone().unwrap_or_default(),
                            "symbol": v.token.symbol.clone().unwrap_or_default(),
                            "decimals": v.token.decimals.clone().map(|d| d.0).unwrap_or_default()
                        },
                        "orderbook": { "id": v.orderbook.id.0 },
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    })
                })
                .collect();

            json!({
                "id": order_hash.to_string(),
                "orderBytes": order_bytes,
                "orderHash": order_hash.to_string(),
                "owner": setup.owner.to_string(),
                "outputs": outputs_json,
                "inputs": inputs_json,
                "orderbook": { "id": setup.orderbook.to_string() },
                "active": true,
                "timestampAdded": "1739448802",
                "meta": null,
                "addEvents": [{
                    "transaction": {
                        "id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                        "from": setup.owner.to_string(),
                        "blockNumber": "1",
                        "timestamp": "1739448802"
                    }
                }],
                "trades": [],
                "removeEvents": []
            })
        }

        fn standard_deposit_amount() -> U256 {
            U256::from(10).pow(U256::from(20))
        }

        async fn fund_standard_two_token_vault(setup: &TestSetup, vault_id: B256) {
            let amount = standard_deposit_amount();
            setup
                .local_evm
                .deposit(setup.owner, setup.token1, amount, 18, vault_id)
                .await;
            setup
                .local_evm
                .deposit(setup.owner, setup.token2, amount, 18, vault_id)
                .await;
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_no_candidates_returns_no_liquidity() {
            let setup = setup_local_evm_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let res = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when no candidates (no vault balance), got: {:?}",
                res
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_happy_path_returns_valid_config() {
            let setup = setup_local_evm_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Should succeed with funded vault and valid order");

            assert_eq!(
                result.orderbook, setup.orderbook,
                "Orderbook address should match"
            );

            let decoded =
                takeOrders3Call::abi_decode(&result.calldata).expect("Should decode calldata");
            let config = decoded.config;

            assert!(
                !config.orders.is_empty(),
                "Should have at least one order in config"
            );

            assert_eq!(
                config.minimumInput,
                Float::zero().unwrap().get_inner(),
                "minimumInput should be zero for Partial mode"
            );

            assert!(
                !result.prices.is_empty(),
                "Should have at least one price in result"
            );

            let expected_ratio = Float::parse("2".to_string()).unwrap();
            assert!(
                result.prices[0].eq(expected_ratio).unwrap(),
                "Price should match expected ratio of 2, got: {:?}",
                result.prices[0].format()
            );

            let zero = Float::zero().unwrap();
            assert!(
                result.effective_price.gt(zero).unwrap(),
                "Effective price should be > 0"
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_min_receive_mode_exact_vs_partial() {
            let setup = setup_local_evm_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result_partial = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Partial mode should succeed");

            let result_exact = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Exact,
                )
                .await
                .expect("Exact mode should succeed");

            let decoded_partial = takeOrders3Call::abi_decode(&result_partial.calldata)
                .expect("Should decode partial calldata");
            let config_partial = decoded_partial.config;

            let decoded_exact = takeOrders3Call::abi_decode(&result_exact.calldata)
                .expect("Should decode exact calldata");
            let config_exact = decoded_exact.config;

            assert_eq!(
                config_partial.maximumInput, config_exact.maximumInput,
                "maximumInput should be the same for both modes"
            );

            assert_eq!(
                config_partial.minimumInput,
                Float::zero().unwrap().get_inner(),
                "minimumInput should be zero for Partial mode"
            );

            assert_eq!(
                config_exact.minimumInput, config_exact.maximumInput,
                "minimumInput should equal maximumInput for Exact mode"
            );

            assert_eq!(
                config_partial.maximumIORatio, config_exact.maximumIORatio,
                "maximumIORatio should be the same for both modes"
            );
        }

        #[tokio::test]
        async fn test_get_take_orders_calldata_wrong_direction_returns_no_liquidity() {
            let setup = setup_local_evm_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config(&setup, "100", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let fake_token = "0xcccccccccccccccccccccccccccccccccccccccc";
            let res = client
                .get_take_orders_calldata(
                    123,
                    fake_token.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await;

            assert!(
                matches!(res, Err(RaindexError::NoLiquidity)),
                "Expected NoLiquidity error when using wrong direction/fake token, got: {:?}",
                res
            );
        }

        #[tokio::test]
        async fn test_min_receive_mode_exact_reverts_when_simulated_buy_cannot_be_met() {
            use alloy::network::TransactionBuilder;
            use alloy::rpc::types::TransactionRequest;
            use alloy::serde::WithOtherFields;

            let setup = setup_local_evm_test().await;
            let sg_server = MockServer::start_async().await;

            let vault_id = B256::from(U256::from(1u64));
            fund_standard_two_token_vault(&setup, vault_id).await;

            let vault1 = create_vault(vault_id, &setup, &setup.token1_sg);
            let vault2 = create_vault(vault_id, &setup, &setup.token2_sg);

            let dotrain = create_dotrain_config(&setup, "50", "2");
            let (order_bytes, order_hash) = deploy_order(&setup, dotrain).await;

            let order_json = create_sg_order_json(
                &setup,
                &order_bytes,
                order_hash,
                vec![vault1.clone(), vault2.clone()],
                vec![vault1.clone(), vault2.clone()],
            );

            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "orders": [order_json]
                    }
                }));
            });

            let yaml = get_minimal_yaml_for_chain(
                123,
                &setup.local_evm.url().to_string(),
                &sg_server.url("/sg"),
                &setup.orderbook.to_string(),
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let result_partial = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Partial,
                )
                .await
                .expect("Partial mode calldata build should succeed");

            let result_exact = client
                .get_take_orders_calldata(
                    123,
                    setup.token1.to_string(),
                    setup.token2.to_string(),
                    "10".to_string(),
                    MinReceiveMode::Exact,
                )
                .await
                .expect("Exact mode calldata build should succeed");

            let decoded_partial = takeOrders3Call::abi_decode(&result_partial.calldata)
                .expect("Should decode partial calldata");
            let config_partial = decoded_partial.config;

            let decoded_exact = takeOrders3Call::abi_decode(&result_exact.calldata)
                .expect("Should decode exact calldata");
            let config_exact = decoded_exact.config;

            assert_eq!(
                config_partial.minimumInput,
                Float::zero().unwrap().get_inner(),
                "Partial mode minimumInput should be zero"
            );
            assert_eq!(
                config_exact.minimumInput, config_exact.maximumInput,
                "Exact mode minimumInput should equal maximumInput"
            );
            assert_eq!(
                config_partial.maximumInput, config_exact.maximumInput,
                "Both modes should have the same maximumInput (simulated total buy)"
            );

            let withdraw_amount =
                Float::from_fixed_decimal(standard_deposit_amount() - U256::from(1), 18)
                    .unwrap()
                    .get_inner();

            let withdraw_tx = setup
                .local_evm
                .orderbook
                .withdraw3(setup.token2, vault_id, withdraw_amount, vec![])
                .from(setup.owner)
                .into_transaction_request();
            setup
                .local_evm
                .send_transaction(withdraw_tx)
                .await
                .expect("Withdraw should succeed");

            let taker = setup.local_evm.signer_wallets[1].default_signer().address();
            let taker_token1_balance = U256::from(10).pow(U256::from(22));
            let token1_contract = setup
                .local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == setup.token1)
                .unwrap();

            token1_contract
                .transfer(taker, taker_token1_balance)
                .from(setup.owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            token1_contract
                .approve(setup.orderbook, taker_token1_balance)
                .from(taker)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();

            let exact_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(result_exact.calldata.to_vec())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let exact_call_result = setup.local_evm.call(exact_tx.clone()).await;
            assert!(
                exact_call_result.is_err(),
                "Exact mode should revert when simulated buy cannot be met, but got: {:?}",
                exact_call_result
            );

            let error_str = format!("{:?}", exact_call_result.unwrap_err());
            assert!(
                error_str.contains("MinimumInput") || error_str.contains("execution reverted"),
                "Error should indicate MinimumInput revert, got: {}",
                error_str
            );

            let partial_tx = WithOtherFields::new(
                TransactionRequest::default()
                    .with_input(result_partial.calldata.to_vec())
                    .with_to(setup.orderbook)
                    .with_from(taker),
            );

            let partial_call_result = setup.local_evm.call(partial_tx.clone()).await;
            assert!(
                partial_call_result.is_ok(),
                "Partial mode should NOT revert (may succeed with smaller totals or be a no-op), but got: {:?}",
                partial_call_result
            );

            let partial_tx_result = setup.local_evm.send_transaction(partial_tx).await;
            assert!(
                partial_tx_result.is_ok(),
                "Partial mode transaction should succeed, but got: {:?}",
                partial_tx_result
            );
        }
    }
}
