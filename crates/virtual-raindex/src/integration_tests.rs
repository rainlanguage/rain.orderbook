#[cfg(test)]
mod integration {
    use crate::{
        cache::StaticCodeCache,
        events::{
            orderbook::{orderbook_event_to_mutations, OrderBookEvent},
            store::{store_event_to_mutation, StoreEvent},
        },
        host::RevmInterpreterHost,
        state::{self, StoreKey, VaultKey},
        Float, OrderRef, QuoteRequest, RaindexMutation, TakeOrder, TakeOrdersConfig,
        TokenDecimalEntry, VaultDelta, VirtualRaindex,
    };
    use alloy::primitives::{Address, Bytes, B256, U256};
    use alloy::providers::{ext::AnvilApi, Provider};
    use alloy::rpc::types::{eth::Filter, BlockNumberOrTag};
    use proptest::prelude::*;
    use rain_interpreter_bindings::IInterpreterStoreV3::Set as StoreSetEvent;
    use rain_interpreter_test_fixtures::{Interpreter, Store};
    use rain_orderbook_bindings::IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearConfigV2, ClearStateChangeV2, ClearV3, DepositV2,
        EvaluableV4, OrderV4, SignedContextV1, TakeOrderConfigV4, TakeOrderV3, TaskV2, IOV2,
    };
    use rain_orderbook_test_fixtures::LocalEvm;
    use rain_orderbook_test_fixtures::Orderbook::{
        self, EvaluableV4 as OnchainEvaluable, OrderConfigV4 as OnchainOrderConfig,
        QuoteV2 as OnchainQuoteV2, TakeOrderConfigV4 as OnchainTakeOrderConfig,
        TakeOrdersConfigV4 as OnchainTakeOrdersConfig, TaskV2 as OnchainTaskV2,
        IOV2 as OnchainIOV2,
    };
    use std::{collections::VecDeque, sync::Arc};
    use tokio::runtime::Runtime;

    const BASE_OUTPUT_DEPOSIT: u64 = 5;
    const MAX_IO_RATIO: &str = "10000000000";
    const ZERO: &str = "0";
    const UNIQUE_STORE_KEY_PRIMARY_BYTES: [u8; 32] = [0x13; 32];
    const SHARED_STORE_KEY_BYTES: [u8; 32] = [0x24; 32];
    const POST_TASK_KEY_PRIMARY_BYTES: [u8; 32] = [0x35; 32];
    const POST_TASK_KEY_SECONDARY_BYTES: [u8; 32] = [0x46; 32];

    type TestVirtualRaindex = VirtualRaindex<StaticCodeCache, RevmInterpreterHost<StaticCodeCache>>;

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_string()).expect("float parse")
    }

    fn float_raw(value: &str) -> B256 {
        parse_float(value).get_inner()
    }

    fn float_to_string(value: &Float) -> String {
        value.format().expect("format float")
    }

    fn amount_to_float(amount: u8) -> Float {
        parse_float(&amount.to_string())
    }

    fn small_b256(value: u64) -> B256 {
        B256::from(U256::from(value))
    }

    fn format_store_key(key: B256) -> String {
        format!("{key:#066x}")
    }

    fn address_to_u256(address: Address) -> U256 {
        U256::from_be_slice(address.into_word().as_slice())
    }

    fn convert_onchain_io(io: &Orderbook::IOV2) -> IOV2 {
        IOV2 {
            token: io.token,
            vaultId: io.vaultId,
        }
    }

    fn convert_onchain_evaluable(evaluable: &Orderbook::EvaluableV4) -> EvaluableV4 {
        EvaluableV4 {
            interpreter: evaluable.interpreter,
            store: evaluable.store,
            bytecode: evaluable.bytecode.clone(),
        }
    }

    fn convert_onchain_order(order: &Orderbook::OrderV4) -> OrderV4 {
        OrderV4 {
            owner: order.owner,
            evaluable: convert_onchain_evaluable(&order.evaluable),
            validInputs: order.validInputs.iter().map(convert_onchain_io).collect(),
            validOutputs: order.validOutputs.iter().map(convert_onchain_io).collect(),
            nonce: order.nonce,
        }
    }

    fn convert_add_order_event(event: &Orderbook::AddOrderV3) -> AddOrderV3 {
        AddOrderV3 {
            sender: event.sender,
            orderHash: event.orderHash,
            order: convert_onchain_order(&event.order),
        }
    }

    fn convert_deposit_event(event: &Orderbook::DepositV2) -> DepositV2 {
        DepositV2 {
            sender: event.sender,
            token: event.token,
            vaultId: event.vaultId,
            depositAmountUint256: event.depositAmountUint256,
        }
    }

    fn convert_store_event(event: &Store::Set) -> StoreSetEvent {
        StoreSetEvent {
            namespace: event.namespace,
            key: event.key,
            value: event.value,
        }
    }

    fn convert_signed_context(context: &Orderbook::SignedContextV1) -> SignedContextV1 {
        SignedContextV1 {
            signer: context.signer,
            context: context.context.clone(),
            signature: context.signature.clone(),
        }
    }

    fn convert_take_order_config(config: &Orderbook::TakeOrderConfigV4) -> TakeOrderConfigV4 {
        TakeOrderConfigV4 {
            order: convert_onchain_order(&config.order),
            inputIOIndex: config.inputIOIndex,
            outputIOIndex: config.outputIOIndex,
            signedContext: config
                .signedContext
                .iter()
                .map(convert_signed_context)
                .collect(),
        }
    }

    fn convert_take_order_event(event: &Orderbook::TakeOrderV3) -> TakeOrderV3 {
        TakeOrderV3 {
            sender: event.sender,
            config: convert_take_order_config(&event.config),
            input: event.input,
            output: event.output,
        }
    }

    fn convert_clear_config(config: &Orderbook::ClearConfigV2) -> ClearConfigV2 {
        ClearConfigV2 {
            aliceInputIOIndex: config.aliceInputIOIndex,
            aliceOutputIOIndex: config.aliceOutputIOIndex,
            bobInputIOIndex: config.bobInputIOIndex,
            bobOutputIOIndex: config.bobOutputIOIndex,
            aliceBountyVaultId: config.aliceBountyVaultId,
            bobBountyVaultId: config.bobBountyVaultId,
        }
    }

    fn convert_clear_event(event: &Orderbook::ClearV3) -> ClearV3 {
        ClearV3 {
            sender: event.sender,
            alice: convert_onchain_order(&event.alice),
            bob: convert_onchain_order(&event.bob),
            clearConfig: convert_clear_config(&event.clearConfig),
        }
    }

    fn convert_after_clear_event(event: &Orderbook::AfterClearV2) -> AfterClearV2 {
        AfterClearV2 {
            sender: event.sender,
            clearStateChange: ClearStateChangeV2 {
                aliceOutput: event.clearStateChange.aliceOutput,
                bobOutput: event.clearStateChange.bobOutput,
                aliceInput: event.clearStateChange.aliceInput,
                bobInput: event.clearStateChange.bobInput,
            },
        }
    }

    enum EventKind {
        OrderBook(OrderBookEventOwned),
        Store(StoreSetEvent),
    }

    enum OrderBookEventOwned {
        Add(AddOrderV3),
        Deposit(DepositV2),
        Take(TakeOrderV3),
        Clear {
            clear: ClearV3,
            state_change: AfterClearV2,
        },
    }

    #[derive(Clone, Copy, Debug)]
    enum OrderTemplate {
        EnvTimestamp,
        VaultBalance,
    }

    impl OrderTemplate {
        fn hashed_literal(self) -> &'static str {
            match self {
                OrderTemplate::EnvTimestamp => "scoped-env",
                OrderTemplate::VaultBalance => "scoped-vault",
            }
        }

        fn hashed_constant(self) -> B256 {
            match self {
                OrderTemplate::EnvTimestamp => B256::from(U256::from(2u64)),
                OrderTemplate::VaultBalance => B256::from(U256::from(3u64)),
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum OrderTarget {
        Primary,
        Secondary,
    }

    impl OrderTarget {
        fn index(self) -> usize {
            match self {
                OrderTarget::Primary => 0,
                OrderTarget::Secondary => 1,
            }
        }

        fn from_index(index: usize) -> Self {
            match index {
                0 => OrderTarget::Primary,
                1 => OrderTarget::Secondary,
                _ => panic!("unsupported order index {index}"),
            }
        }
    }

    struct DeployOrderParams {
        owner: Address,
        template: OrderTemplate,
        unique_store_key: Option<B256>,
        shared_store_key: Option<B256>,
        post_task_key: B256,
        input_token: Address,
        output_token: Address,
        input_vault_id: B256,
        output_vault_id: B256,
    }

    #[derive(Clone, Debug)]
    enum Action {
        Deposit(OrderTarget, u8),
        Take(OrderTarget, u8),
        Quote(OrderTarget),
        AdvanceTime(u16),
    }

    fn action_strategy() -> impl Strategy<Value = Vec<Action>> {
        let order = prop_oneof![Just(OrderTarget::Primary), Just(OrderTarget::Secondary)];
        let deposit =
            (order.clone(), 1u8..=4).prop_map(|(target, amount)| Action::Deposit(target, amount));
        let take =
            (order.clone(), 1u8..=3).prop_map(|(target, amount)| Action::Take(target, amount));
        let quote = order.clone().prop_map(Action::Quote);
        let advance = (1u16..=600).prop_map(Action::AdvanceTime);

        prop::collection::vec(prop_oneof![deposit, take.clone(), quote, advance], 6..=12)
            .prop_filter(
                "available actions must include at least one take",
                move |actions| actions.iter().any(|a| matches!(a, Action::Take(_, _))),
            )
    }

    struct OrderState {
        onchain_order: Orderbook::OrderV4,
        order_hash: B256,
        input_token: Address,
        output_token: Address,
        input_vault_id: B256,
        output_vault_id: B256,
        input_key: VaultKey,
        output_key: VaultKey,
        unique_store_key: Option<B256>,
        shared_store_key: Option<B256>,
        hashed_store_key: Option<B256>,
        hashed_expected_value: B256,
        post_task_key: B256,
        post_task_expected_value: B256,
    }

    impl OrderState {
        fn tracked_store_keys(&self) -> Vec<(B256, Option<B256>)> {
            let mut keys: Vec<(B256, Option<B256>)> = Vec::new();

            let mut push_key = |key: Option<B256>, expected: Option<B256>| {
                if let Some(actual_key) = key {
                    if !keys.iter().any(|(existing, _)| *existing == actual_key) {
                        keys.push((actual_key, expected));
                    }
                }
            };

            push_key(self.unique_store_key, None);
            push_key(self.shared_store_key, None);
            push_key(self.hashed_store_key, Some(self.hashed_expected_value));

            if !keys
                .iter()
                .any(|(existing, _)| *existing == self.post_task_key)
            {
                keys.push((self.post_task_key, Some(self.post_task_expected_value)));
            }

            keys
        }
    }

    struct Harness {
        local_evm: LocalEvm,
        raindex: TestVirtualRaindex,
        owner: Address,
        secondary_owner: Address,
        store_address: Address,
        orderbook_address: Address,
        interpreter_address: Address,
        orders: Vec<OrderState>,
    }

    impl Harness {
        async fn new() -> Self {
            let local_evm = LocalEvm::new_with_tokens(2).await;
            let owner = local_evm.anvil.addresses()[0];
            let secondary_owner = local_evm.anvil.addresses()[1];
            let orderbook_address = *local_evm.orderbook.address();
            let interpreter_address = *local_evm.interpreter.address();
            let store_address = *local_evm.store.address();

            let cache = Arc::new(StaticCodeCache::default());
            cache.upsert_interpreter(interpreter_address, Interpreter::DEPLOYED_BYTECODE.as_ref());
            cache.upsert_store(store_address, Store::DEPLOYED_BYTECODE.as_ref());
            let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
            let raindex = VirtualRaindex::new(orderbook_address, cache.clone(), host);

            let mut harness = Self {
                local_evm,
                raindex,
                owner,
                secondary_owner,
                store_address,
                orderbook_address,
                interpreter_address,
                orders: Vec::new(),
            };

            harness.initialise_orders().await;
            harness
        }

        async fn initialise_orders(&mut self) {
            let input_token = *self.local_evm.tokens[0].address();
            let output_token = *self.local_evm.tokens[1].address();

            let transfer_amount = parse_float("20");
            let transfer_amount_wei = transfer_amount
                .to_fixed_decimal(18)
                .expect("transfer amount to fixed");
            for token in &self.local_evm.tokens {
                self.local_evm
                    .send_transaction(
                        token
                            .transfer(self.secondary_owner, transfer_amount_wei)
                            .from(self.owner)
                            .into_transaction_request(),
                    )
                    .await
                    .expect("transfer tokens to secondary owner");
            }

            self.local_evm
                .send_transaction(
                    self.local_evm.tokens[1]
                        .approve(self.orderbook_address, U256::MAX)
                        .from(self.owner)
                        .into_transaction_request(),
                )
                .await
                .expect("approve output token");

            self.local_evm
                .send_transaction(
                    self.local_evm.tokens[0]
                        .approve(self.orderbook_address, U256::MAX)
                        .from(self.owner)
                        .into_transaction_request(),
                )
                .await
                .expect("approve input token");

            for token in &self.local_evm.tokens {
                self.local_evm
                    .send_transaction(
                        token
                            .approve(self.orderbook_address, U256::MAX)
                            .from(self.secondary_owner)
                            .into_transaction_request(),
                    )
                    .await
                    .expect("approve token for secondary owner");
            }

            self.raindex
                .apply_mutations(&[RaindexMutation::SetTokenDecimals {
                    entries: vec![
                        TokenDecimalEntry {
                            token: input_token,
                            decimals: 18,
                        },
                        TokenDecimalEntry {
                            token: output_token,
                            decimals: 18,
                        },
                    ],
                }])
                .expect("virtual decimals");

            let unique_primary = B256::from(UNIQUE_STORE_KEY_PRIMARY_BYTES);
            let post_primary = B256::from(POST_TASK_KEY_PRIMARY_BYTES);
            let post_secondary = B256::from(POST_TASK_KEY_SECONDARY_BYTES);
            let shared_key = B256::from(SHARED_STORE_KEY_BYTES);

            self.deploy_order(DeployOrderParams {
                owner: self.owner,
                template: OrderTemplate::EnvTimestamp,
                unique_store_key: Some(unique_primary),
                shared_store_key: Some(shared_key),
                post_task_key: post_primary,
                input_token,
                output_token,
                input_vault_id: B256::from([1u8; 32]),
                output_vault_id: B256::from([2u8; 32]),
            })
            .await;

            self.deploy_order(DeployOrderParams {
                owner: self.secondary_owner,
                template: OrderTemplate::VaultBalance,
                unique_store_key: None,
                shared_store_key: Some(shared_key),
                post_task_key: post_secondary,
                input_token: output_token,
                output_token: input_token,
                input_vault_id: B256::from([3u8; 32]),
                output_vault_id: B256::from([4u8; 32]),
            })
            .await;

            for target in [OrderTarget::Primary, OrderTarget::Secondary] {
                self.deposit_output(target, amount_to_float(BASE_OUTPUT_DEPOSIT as u8))
                    .await;
            }

            self.assert_all_balances_synced().await;
            self.assert_all_store_synced().await;
        }

        async fn deploy_order(&mut self, params: DeployOrderParams) {
            let DeployOrderParams {
                owner,
                template,
                unique_store_key,
                shared_store_key,
                post_task_key,
                input_token,
                output_token,
                input_vault_id,
                output_vault_id,
            } = params;
            let subparser = *self.local_evm.orderbook_subparser.address();
            let rain_src = build_rainlang(template, unique_store_key, shared_store_key, subparser);
            let bytecode = compile_rain(&self.local_evm, rain_src).await;

            let order = OrderV4 {
                owner,
                evaluable: EvaluableV4 {
                    interpreter: self.interpreter_address,
                    store: self.store_address,
                    bytecode: bytecode.clone(),
                },
                validInputs: vec![IOV2 {
                    token: input_token,
                    vaultId: input_vault_id,
                }],
                validOutputs: vec![IOV2 {
                    token: output_token,
                    vaultId: output_vault_id,
                }],
                nonce: B256::from(U256::from(self.orders.len() as u64 + 1)),
            };

            let onchain_evaluable = OnchainEvaluable {
                interpreter: self.interpreter_address,
                store: self.store_address,
                bytecode: bytecode.clone(),
            };
            let onchain_config = OnchainOrderConfig {
                evaluable: onchain_evaluable,
                validInputs: order
                    .validInputs
                    .iter()
                    .map(|io| OnchainIOV2 {
                        token: io.token,
                        vaultId: io.vaultId,
                    })
                    .collect(),
                validOutputs: order
                    .validOutputs
                    .iter()
                    .map(|io| OnchainIOV2 {
                        token: io.token,
                        vaultId: io.vaultId,
                    })
                    .collect(),
                nonce: order.nonce,
                secret: B256::ZERO,
                meta: Bytes::new(),
            };

            let post_task_source = build_post_task_source(template, post_task_key, subparser);
            let post_task_bytecode = compile_rain(&self.local_evm, post_task_source).await;

            let virtual_tasks = vec![TaskV2 {
                evaluable: EvaluableV4 {
                    interpreter: self.interpreter_address,
                    store: self.store_address,
                    bytecode: post_task_bytecode.clone(),
                },
                signedContext: Vec::new(),
            }];

            self.raindex
                .add_order(order.clone(), virtual_tasks.clone())
                .expect("virtual add order");

            let onchain_task = OnchainTaskV2 {
                evaluable: OnchainEvaluable {
                    interpreter: self.interpreter_address,
                    store: self.store_address,
                    bytecode: post_task_bytecode.clone(),
                },
                signedContext: Vec::new(),
            };

            self.local_evm
                .send_transaction(
                    self.local_evm
                        .orderbook
                        .addOrder3(onchain_config.clone(), vec![onchain_task])
                        .from(owner)
                        .into_transaction_request(),
                )
                .await
                .expect("add order onchain");

            let order_hash = state::order_hash(&order);
            let onchain_order = Orderbook::OrderV4 {
                owner: order.owner,
                evaluable: onchain_config.evaluable.clone(),
                validInputs: onchain_config.validInputs.clone(),
                validOutputs: onchain_config.validOutputs.clone(),
                nonce: order.nonce,
            };

            let order_state = OrderState {
                onchain_order,
                order_hash,
                input_token,
                output_token,
                input_vault_id,
                output_vault_id,
                input_key: VaultKey::new(order.owner, input_token, input_vault_id),
                output_key: VaultKey::new(order.owner, output_token, output_vault_id),
                unique_store_key,
                shared_store_key,
                hashed_store_key: None,
                hashed_expected_value: template.hashed_constant(),
                post_task_key,
                post_task_expected_value: small_b256(1),
            };

            self.orders.push(order_state);
            self.assert_store_synced_for(OrderTarget::from_index(self.orders.len() - 1))
                .await;
        }

        async fn sync_env_with_chain(&mut self) {
            let block_number = self
                .local_evm
                .provider
                .get_block_number()
                .await
                .expect("block number");
            let block = self
                .local_evm
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(block_number))
                .await
                .expect("block query")
                .expect("latest block");

            let block_number_u64: u64 = block_number;
            let timestamp_u64 = block.header.timestamp;

            self.raindex
                .apply_mutations(&[RaindexMutation::SetEnv {
                    block_number: Some(block_number_u64),
                    timestamp: Some(timestamp_u64),
                }])
                .expect("virtual set env");
        }

        async fn deposit_output(&mut self, target: OrderTarget, amount: Float) {
            let order = &self.orders[target.index()];
            let raw_amount = amount.get_inner();
            let amount_wei = amount.to_fixed_decimal(18).expect("float to fixed");

            self.local_evm
                .send_transaction(
                    self.local_evm
                        .tokens
                        .iter()
                        .find(|token| *token.address() == order.output_token)
                        .expect("output token instance")
                        .approve(self.orderbook_address, amount_wei)
                        .from(order.onchain_order.owner)
                        .into_transaction_request(),
                )
                .await
                .expect("refresh output allowance");

            self.local_evm
                .send_transaction(
                    self.local_evm
                        .orderbook
                        .deposit3(
                            order.output_token,
                            order.output_vault_id,
                            raw_amount,
                            Vec::<OnchainTaskV2>::new(),
                        )
                        .from(order.onchain_order.owner)
                        .into_transaction_request(),
                )
                .await
                .expect("deposit onchain");

            self.raindex
                .apply_mutations(&[RaindexMutation::VaultDeltas {
                    deltas: vec![VaultDelta {
                        owner: order.onchain_order.owner,
                        token: order.output_token,
                        vault_id: order.output_vault_id,
                        delta: amount,
                    }],
                }])
                .expect("virtual deposit");

            self.sync_env_with_chain().await;
        }

        async fn advance_time(&mut self, seconds: u64) {
            self.local_evm
                .provider
                .anvil_increase_time(seconds)
                .await
                .expect("increase time");
            self.local_evm
                .provider
                .anvil_mine(Some(1), None)
                .await
                .expect("mine block");
            self.sync_env_with_chain().await;
        }

        async fn assert_quotes_match(&mut self, target: OrderTarget) {
            self.sync_env_with_chain().await;
            let order = &self.orders[target.index()];

            let quote_return = self
                .local_evm
                .orderbook
                .quote2(OnchainQuoteV2 {
                    order: order.onchain_order.clone(),
                    inputIOIndex: U256::ZERO,
                    outputIOIndex: U256::ZERO,
                    signedContext: Vec::new(),
                })
                .call()
                .await
                .expect("quote onchain");
            assert!(quote_return._0);

            let virtual_quote = self
                .raindex
                .quote(QuoteRequest::new(
                    OrderRef::ByHash(order.order_hash),
                    0,
                    0,
                    self.owner,
                ))
                .expect("virtual quote");

            assert_eq!(
                float_to_string(&Float::from_raw(quote_return._1)),
                float_to_string(&virtual_quote.output_max),
            );
            assert_eq!(
                float_to_string(&Float::from_raw(quote_return._2)),
                float_to_string(&virtual_quote.io_ratio),
            );
        }

        async fn take(&mut self, target: OrderTarget, amount: Float) {
            self.assert_quotes_match(target).await;
            let onchain_config = self.onchain_take_config(target, amount);

            self.local_evm
                .send_transaction(
                    self.local_evm
                        .orderbook
                        .takeOrders3(onchain_config)
                        .from(self.owner)
                        .into_transaction_request(),
                )
                .await
                .expect("take onchain");

            self.sync_env_with_chain().await;

            let outcome = self
                .raindex
                .take_orders_and_apply_state(self.virtual_take_config(target, amount))
                .expect("virtual take");

            assert!(
                outcome.warnings.is_empty(),
                "virtual take produced warnings: {:?}",
                outcome.warnings
            );
        }

        async fn clear(&mut self) {
            self.sync_env_with_chain().await;

            let alice = self.orders[0].onchain_order.clone();
            let bob = self.orders[1].onchain_order.clone();

            let clear_config = Orderbook::ClearConfigV2 {
                aliceInputIOIndex: U256::ZERO,
                aliceOutputIOIndex: U256::ZERO,
                bobInputIOIndex: U256::ZERO,
                bobOutputIOIndex: U256::ZERO,
                aliceBountyVaultId: alice.validOutputs[0].vaultId,
                bobBountyVaultId: bob.validOutputs[0].vaultId,
            };

            let tx = self
                .local_evm
                .orderbook
                .clear3(
                    alice.clone(),
                    bob.clone(),
                    clear_config.clone(),
                    vec![],
                    vec![],
                )
                .from(self.owner)
                .into_transaction_request();

            let receipt = self
                .local_evm
                .send_transaction(tx)
                .await
                .expect("clear onchain");

            let mut clear_log: Option<Orderbook::ClearV3> = None;
            let mut after_clear_log: Option<Orderbook::AfterClearV2> = None;
            let mut store_events: Vec<Store::Set> = Vec::new();

            for log in receipt.inner.inner.logs() {
                if let Ok(decoded) = log.log_decode::<Orderbook::ClearV3>() {
                    clear_log = Some(decoded.inner.data);
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Orderbook::AfterClearV2>() {
                    after_clear_log = Some(decoded.inner.data);
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Store::Set>() {
                    store_events.push(decoded.inner.data);
                }
            }

            let clear = clear_log.expect("clear event emitted");
            let after = after_clear_log.expect("after clear event emitted");

            let converted_clear = convert_clear_event(&clear);
            let converted_after = convert_after_clear_event(&after);

            let clear_mutations = orderbook_event_to_mutations(OrderBookEvent::Clear {
                clear: &converted_clear,
                state_change: &converted_after,
            })
            .expect("convert clear mutations");

            self.raindex
                .apply_mutations(&clear_mutations)
                .expect("apply clear mutations");

            for store_event in store_events {
                let converted = convert_store_event(&store_event);
                let mutation = store_event_to_mutation(StoreEvent {
                    store: self.store_address,
                    data: &converted,
                });
                self.raindex
                    .apply_mutations(&[mutation])
                    .expect("apply store mutation from clear");
            }

            self.assert_all_balances_synced().await;
            self.assert_all_store_synced().await;
        }

        async fn assert_balances_synced_for(&self, target: OrderTarget) {
            let order = &self.orders[target.index()];
            let snapshot = self.raindex.snapshot();
            let virtual_input = snapshot
                .vault_balances
                .get(&order.input_key)
                .cloned()
                .unwrap_or_default();
            let virtual_output = snapshot
                .vault_balances
                .get(&order.output_key)
                .cloned()
                .unwrap_or_default();

            let onchain_input = self
                .local_evm
                .orderbook
                .vaultBalance2(
                    order.onchain_order.owner,
                    order.input_token,
                    order.input_vault_id,
                )
                .call()
                .await
                .expect("onchain input vault");
            let onchain_output = self
                .local_evm
                .orderbook
                .vaultBalance2(
                    order.onchain_order.owner,
                    order.output_token,
                    order.output_vault_id,
                )
                .call()
                .await
                .expect("onchain output vault");

            assert_eq!(
                float_to_string(&Float::from_raw(onchain_input)),
                float_to_string(&virtual_input),
            );
            assert_eq!(
                float_to_string(&Float::from_raw(onchain_output)),
                float_to_string(&virtual_output),
            );
        }

        async fn assert_store_synced_for(&mut self, target: OrderTarget) {
            let idx = target.index();
            let order = self.orders.get_mut(idx).expect("order state for target");
            let namespace = address_to_u256(order.onchain_order.owner);
            let fqn = state::derive_fqn(namespace, self.orderbook_address);
            let fqn_u256 = U256::from_be_slice(fqn.as_slice());

            let snapshot = self.raindex.snapshot();

            if order.hashed_store_key.is_none() {
                let discovered = snapshot.store.iter().find_map(|(store_key, value)| {
                    if store_key.store == self.store_address
                        && store_key.fqn == fqn
                        && *value == order.hashed_expected_value
                    {
                        Some(store_key.key)
                    } else {
                        None
                    }
                });
                if let Some(key) = discovered {
                    order.hashed_store_key = Some(key);
                }
            }

            for (key, expected) in order.tracked_store_keys() {
                let store_key = StoreKey::new(self.store_address, fqn, key);
                let virtual_value = snapshot
                    .store
                    .get(&store_key)
                    .cloned()
                    .unwrap_or(B256::ZERO);

                let onchain_value = self
                    .local_evm
                    .store
                    .get(fqn_u256, key)
                    .call()
                    .await
                    .expect("store get");

                assert_eq!(virtual_value, onchain_value);

                if let Some(expected_value) = expected {
                    assert_eq!(
                        virtual_value,
                        expected_value,
                        "store key {} expected constant {}",
                        format_store_key(key),
                        format_store_key(expected_value)
                    );
                }
            }
        }

        async fn assert_all_balances_synced(&self) {
            for idx in 0..self.orders.len() {
                self.assert_balances_synced_for(OrderTarget::from_index(idx))
                    .await;
            }
        }

        async fn assert_all_store_synced(&mut self) {
            for idx in 0..self.orders.len() {
                self.assert_store_synced_for(OrderTarget::from_index(idx))
                    .await;
            }
        }

        async fn run_actions(&mut self, actions: &[Action]) {
            self.sync_env_with_chain().await;
            self.assert_all_balances_synced().await;
            self.assert_all_store_synced().await;

            for action in actions {
                match action {
                    Action::Deposit(order, amount) => {
                        self.deposit_output(*order, amount_to_float(*amount)).await;
                    }
                    Action::Take(order, amount) => {
                        self.take(*order, amount_to_float(*amount)).await;
                    }
                    Action::Quote(order) => {
                        self.assert_quotes_match(*order).await;
                    }
                    Action::AdvanceTime(seconds) => {
                        self.advance_time(u64::from(*seconds)).await;
                    }
                }

                self.assert_all_balances_synced().await;
                self.assert_all_store_synced().await;

                if let Action::Take(order, _) = action {
                    assert!(
                        self.orders[order.index()].hashed_store_key.is_some(),
                        "hashed key should be discovered after take for {:?}",
                        order
                    );
                }
            }
        }

        fn virtual_take_config(&self, target: OrderTarget, amount: Float) -> TakeOrdersConfig {
            let order = &self.orders[target.index()];
            TakeOrdersConfig {
                orders: vec![TakeOrder {
                    order: OrderRef::ByHash(order.order_hash),
                    input_io_index: 0,
                    output_io_index: 0,
                    signed_context: Vec::new(),
                }],
                minimum_input: parse_float(ZERO),
                maximum_input: amount,
                maximum_io_ratio: parse_float(MAX_IO_RATIO),
                taker: self.owner,
                data: Vec::new(),
            }
        }

        fn onchain_take_config(
            &self,
            target: OrderTarget,
            amount: Float,
        ) -> OnchainTakeOrdersConfig {
            let order = &self.orders[target.index()];
            OnchainTakeOrdersConfig {
                orders: vec![OnchainTakeOrderConfig {
                    order: order.onchain_order.clone(),
                    inputIOIndex: U256::ZERO,
                    outputIOIndex: U256::ZERO,
                    signedContext: Vec::new(),
                }],
                maximumIORatio: float_raw(MAX_IO_RATIO),
                maximumInput: amount.get_inner(),
                minimumInput: float_raw(ZERO),
                data: Bytes::new(),
            }
        }
    }

    fn build_rainlang(
        template: OrderTemplate,
        unique_store_key: Option<B256>,
        shared_store_key: Option<B256>,
        subparser: Address,
    ) -> String {
        let mut script = format!("using-words-from {subparser}\n\n");

        match template {
            OrderTemplate::EnvTimestamp => {
                script.push_str("/* 0. calculate-io */\n_ _: 1 1;\n\n/* 1. handle-io */\n");
            }
            OrderTemplate::VaultBalance => {
                script.push_str(
                    "/* 0. calculate-io */\n_ _: 1 context<3 3>();\n\n/* 1. handle-io */\n",
                );
            }
        }

        let mut handle_lines: Vec<String> = Vec::new();

        let value_for_template = |template| match template {
            OrderTemplate::EnvTimestamp => "block-timestamp()",
            OrderTemplate::VaultBalance => "context<3 3>()",
        };

        if let Some(key) = unique_store_key {
            handle_lines.push(format!(
                ":set({} {})",
                format_store_key(key),
                value_for_template(template)
            ));
        }

        if let Some(key) = shared_store_key {
            handle_lines.push(format!(
                ":set({} {})",
                format_store_key(key),
                value_for_template(template)
            ));
        }

        for (idx, line) in handle_lines.iter().enumerate() {
            if idx + 1 == handle_lines.len() {
                script.push_str(&format!("{};\n", line));
            } else {
                script.push_str(&format!("{},\n", line));
            }
        }

        script
    }

    fn build_post_task_source(template: OrderTemplate, key: B256, subparser: Address) -> String {
        format!(
            "using-words-from {subparser}\n:set({} 0x01),\n:set(hash(order-hash() \"{}\") {});",
            format_store_key(key),
            template.hashed_literal(),
            format_store_key(template.hashed_constant())
        )
    }

    async fn compile_rain(local_evm: &LocalEvm, source: String) -> Bytes {
        local_evm
            .deployer
            .parse2(Bytes::from(source.into_bytes()))
            .call()
            .await
            .expect("parse2")
    }

    #[test]
    fn event_ingestion_recreates_virtual_state() {
        let runtime = Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let mut harness = Harness::new().await;
            let unit = amount_to_float(1);
            harness.take(OrderTarget::Primary, unit).await;
            harness.clear().await;
            let expected_snapshot = harness.raindex.snapshot();

            let cache = Arc::new(StaticCodeCache::default());
            cache.upsert_interpreter(
                harness.interpreter_address,
                Interpreter::DEPLOYED_BYTECODE.as_ref(),
            );
            cache.upsert_store(harness.store_address, Store::DEPLOYED_BYTECODE.as_ref());
            let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
            let mut replay = VirtualRaindex::new(harness.orderbook_address, cache, host);

            replay
                .apply_mutations(&[RaindexMutation::SetEnv {
                    block_number: Some(expected_snapshot.env.block_number),
                    timestamp: Some(expected_snapshot.env.timestamp),
                }])
                .expect("set env from snapshot");

            if !expected_snapshot.token_decimals.is_empty() {
                let entries = expected_snapshot
                    .token_decimals
                    .iter()
                    .map(|(&token, &decimals)| TokenDecimalEntry { token, decimals })
                    .collect::<Vec<_>>();
                replay
                    .apply_mutations(&[RaindexMutation::SetTokenDecimals { entries }])
                    .expect("set token decimals");
            }

            let mut combined_events: Vec<(u64, u64, EventKind)> = Vec::new();

            let orderbook_filter = Filter::new()
                .address(harness.orderbook_address)
                .from_block(BlockNumberOrTag::Earliest)
                .to_block(BlockNumberOrTag::Latest);
            let orderbook_logs = harness
                .local_evm
                .provider
                .get_logs(&orderbook_filter)
                .await
                .expect("fetch orderbook logs");

            let mut pending_clears: VecDeque<Orderbook::ClearV3> = VecDeque::new();
            for log in orderbook_logs {
                let block = log
                    .block_number
                    .expect("orderbook log should include block number");
                let index = log
                    .log_index
                    .expect("orderbook log should include log index");

                if let Ok(decoded) = log.log_decode::<Orderbook::ClearV3>() {
                    pending_clears.push_back(decoded.into_inner().data);
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Orderbook::AfterClearV2>() {
                    let clear_event = pending_clears.pop_front().expect("matching clear event");
                    let converted_clear = convert_clear_event(&clear_event);
                    let converted_after = convert_after_clear_event(&decoded.inner);
                    combined_events.push((
                        block,
                        index,
                        EventKind::OrderBook(OrderBookEventOwned::Clear {
                            clear: converted_clear,
                            state_change: converted_after,
                        }),
                    ));
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Orderbook::TakeOrderV3>() {
                    let converted = convert_take_order_event(&decoded.inner);
                    combined_events.push((
                        block,
                        index,
                        EventKind::OrderBook(OrderBookEventOwned::Take(converted)),
                    ));
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Orderbook::AddOrderV3>() {
                    let converted = convert_add_order_event(&decoded.inner);
                    combined_events.push((
                        block,
                        index,
                        EventKind::OrderBook(OrderBookEventOwned::Add(converted)),
                    ));
                    continue;
                }

                if let Ok(decoded) = log.log_decode::<Orderbook::DepositV2>() {
                    let converted = convert_deposit_event(&decoded.inner);
                    combined_events.push((
                        block,
                        index,
                        EventKind::OrderBook(OrderBookEventOwned::Deposit(converted)),
                    ));
                    continue;
                }
            }

            assert!(
                pending_clears.is_empty(),
                "unmatched clear events in log replay"
            );

            let store_filter = Filter::new()
                .address(harness.store_address)
                .from_block(BlockNumberOrTag::Earliest)
                .to_block(BlockNumberOrTag::Latest);
            let store_logs = harness
                .local_evm
                .provider
                .get_logs(&store_filter)
                .await
                .expect("fetch store logs");

            for log in store_logs {
                let block = log
                    .block_number
                    .expect("store log should include block number");
                let index = log.log_index.expect("store log should include log index");
                if let Ok(decoded) = log.log_decode::<Store::Set>() {
                    let converted = convert_store_event(&decoded.inner);
                    combined_events.push((block, index, EventKind::Store(converted)));
                }
            }

            combined_events.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
            assert!(
                combined_events.iter().any(|(_, _, kind)| matches!(
                    kind,
                    EventKind::OrderBook(OrderBookEventOwned::Add(_))
                )),
                "expected add order events in log replay"
            );
            assert!(
                combined_events.iter().any(|(_, _, kind)| matches!(
                    kind,
                    EventKind::OrderBook(OrderBookEventOwned::Take(_))
                )),
                "expected take order events in log replay"
            );
            assert!(
                combined_events.iter().any(|(_, _, kind)| matches!(
                    kind,
                    EventKind::OrderBook(OrderBookEventOwned::Clear { .. })
                )),
                "expected clear events in log replay"
            );

            for (_, _, kind) in combined_events {
                match kind {
                    EventKind::OrderBook(OrderBookEventOwned::Add(event)) => {
                        let mutations =
                            orderbook_event_to_mutations(OrderBookEvent::AddOrder(&event))
                                .expect("convert add order");
                        replay
                            .apply_mutations(&mutations)
                            .expect("apply add order mutations");
                    }
                    EventKind::OrderBook(OrderBookEventOwned::Deposit(event)) => {
                        let decimals = expected_snapshot.token_decimals.get(&event.token).copied();
                        let mutations = orderbook_event_to_mutations(OrderBookEvent::Deposit {
                            event: &event,
                            decimals,
                        })
                        .expect("convert deposit");
                        replay
                            .apply_mutations(&mutations)
                            .expect("apply deposit mutation");
                    }
                    EventKind::OrderBook(OrderBookEventOwned::Take(event)) => {
                        let mutations =
                            orderbook_event_to_mutations(OrderBookEvent::TakeOrder(&event))
                                .expect("convert take order");
                        replay
                            .apply_mutations(&mutations)
                            .expect("apply take order mutation");
                    }
                    EventKind::OrderBook(OrderBookEventOwned::Clear {
                        clear,
                        state_change,
                    }) => {
                        let mutations = orderbook_event_to_mutations(OrderBookEvent::Clear {
                            clear: &clear,
                            state_change: &state_change,
                        })
                        .expect("convert clear event");
                        replay
                            .apply_mutations(&mutations)
                            .expect("apply clear mutation");
                    }
                    EventKind::Store(event) => {
                        let mutation = store_event_to_mutation(StoreEvent {
                            store: harness.store_address,
                            data: &event,
                        });
                        replay
                            .apply_mutations(&[mutation])
                            .expect("apply store mutation");
                    }
                }
            }

            let actual_snapshot = replay.snapshot();
            assert_eq!(
                actual_snapshot.orders, expected_snapshot.orders,
                "orders reconstructed from events"
            );
            assert_eq!(
                actual_snapshot.vault_balances.len(),
                expected_snapshot.vault_balances.len(),
                "vault balance entry counts should match"
            );
            for (key, expected_value) in &expected_snapshot.vault_balances {
                let actual = actual_snapshot
                    .vault_balances
                    .get(key)
                    .unwrap_or_else(|| panic!("missing vault entry for {key:?}"));
                let actual_str = actual.format().expect("format actual float");
                let expected_str = expected_value.format().expect("format expected float");
                assert_eq!(
                    actual_str, expected_str,
                    "vault balance mismatch for {key:?}"
                );
            }
            assert_eq!(
                actual_snapshot.store, expected_snapshot.store,
                "store state reconstructed"
            );
            assert_eq!(
                actual_snapshot.token_decimals, expected_snapshot.token_decimals,
                "token decimals preserved"
            );
        });
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn virtual_and_onchain_stay_in_sync() {
        let mut harness = Harness::new().await;

        let unit = amount_to_float(1);
        harness.take(OrderTarget::Primary, unit).await;
        harness.assert_all_balances_synced().await;
        harness.assert_all_store_synced().await;

        harness
            .deposit_output(OrderTarget::Secondary, amount_to_float(2))
            .await;
        harness.advance_time(30).await;
        harness.assert_quotes_match(OrderTarget::Primary).await;
        harness.assert_quotes_match(OrderTarget::Secondary).await;
        harness.take(OrderTarget::Secondary, unit).await;
        harness.assert_all_balances_synced().await;
        harness.assert_all_store_synced().await;

        assert!(
            harness
                .orders
                .iter()
                .all(|order| order.hashed_store_key.is_some()),
            "hashed store keys should be discovered after taking both orders"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 6,
            max_shrink_iters: 32,
            .. ProptestConfig::default()
        })]
        #[test]
        fn virtual_and_onchain_env_actions_remain_in_sync(actions in action_strategy()) {
            let runtime = Runtime::new().expect("runtime");
            runtime.block_on(async move {
                let mut harness = Harness::new().await;
                harness.run_actions(&actions).await;
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 6,
            max_shrink_iters: 32,
            .. ProptestConfig::default()
        })]
        #[test]
        fn virtual_and_onchain_vault_actions_remain_in_sync(actions in action_strategy().prop_filter(
            "actions must include a take for each order",
            |actions: &Vec<Action>| {
                let mut seen = [false, false];
                for action in actions {
                    if let Action::Take(order, _) = action {
                        seen[order.index()] = true;
                    }
                }
                seen.iter().all(|flag| *flag)
            }
        )) {
            let runtime = Runtime::new().expect("runtime");
            runtime.block_on(async move {
                let mut harness = Harness::new().await;
                harness.run_actions(&actions).await;
            });
        }
    }
}
