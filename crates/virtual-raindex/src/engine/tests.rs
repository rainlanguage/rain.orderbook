use super::{
    context::{
        IOContext, CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_VAULT_INPUTS_COLUMN,
        CONTEXT_VAULT_OUTPUTS_COLUMN,
    },
    OrderRef, QuoteRequest, StoreOverride, TakeOrder, TakeOrderWarning, TakeOrdersConfig,
    VirtualRaindex,
};
use crate::{
    cache::{CodeCache, StaticCodeCache},
    error::{RaindexError, Result},
    host::{self, InterpreterHost},
    state::{
        Env, RaindexMutation, StoreKey, StoreKeyValue, StoreSet, TokenDecimalEntry, VaultDelta,
    },
    store::{address_to_u256, derive_fqn},
    RevmInterpreterHost, VaultKey,
};
use alloy::primitives::{Address, Bytes, B256, U256};
use rain_interpreter_bindings::IInterpreterV4::EvalV4;
use rain_interpreter_test_fixtures::{Interpreter, LocalEvm, Store};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, TaskV2, IOV2};
use rain_orderbook_common::utils::order_hash::order_hash;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct NullCache;

impl CodeCache for NullCache {
    fn interpreter(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
        None
    }

    fn store(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
        None
    }

    fn ensure_artifacts(&self, _order: &OrderV4) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct NullInterpreter;

impl host::InterpreterHost for NullInterpreter {
    fn eval4(
        &self,
        _interpreter: Address,
        _eval: &EvalV4,
        _store_snapshot: &HashMap<StoreKey, B256>,
        _env: Env,
    ) -> Result<host::EvalOutcome> {
        Err(RaindexError::Unimplemented("interpreter host"))
    }
}

fn new_raindex() -> VirtualRaindex<NullCache, NullInterpreter> {
    let cache = Arc::new(NullCache);
    let interpreter = Arc::new(NullInterpreter);
    VirtualRaindex::new(Address::ZERO, cache, interpreter)
}

struct RecordingHost {
    outcome: host::EvalOutcome,
    evals: Mutex<Vec<EvalV4>>,
    snapshots: Mutex<Vec<HashMap<StoreKey, B256>>>,
    scripted: Mutex<Vec<host::EvalOutcome>>,
}

impl RecordingHost {
    fn new(outcome: host::EvalOutcome) -> Self {
        Self {
            outcome,
            evals: Mutex::new(Vec::new()),
            snapshots: Mutex::new(Vec::new()),
            scripted: Mutex::new(Vec::new()),
        }
    }

    fn last_eval(&self) -> Option<EvalV4> {
        self.evals.lock().unwrap().last().cloned()
    }

    fn last_snapshot(&self) -> Option<HashMap<StoreKey, B256>> {
        self.snapshots.lock().unwrap().last().cloned()
    }

    fn push_outcome(&self, outcome: host::EvalOutcome) {
        self.scripted.lock().unwrap().push(outcome);
    }
}

impl host::InterpreterHost for RecordingHost {
    fn eval4(
        &self,
        _interpreter: Address,
        eval: &EvalV4,
        store_snapshot: &HashMap<StoreKey, B256>,
        _env: Env,
    ) -> Result<host::EvalOutcome> {
        self.evals.lock().unwrap().push(eval.clone());
        self.snapshots.lock().unwrap().push(store_snapshot.clone());
        let mut scripted = self.scripted.lock().unwrap();
        if !scripted.is_empty() {
            Ok(scripted.remove(0))
        } else {
            Ok(self.outcome.clone())
        }
    }
}

fn test_order() -> OrderV4 {
    OrderV4 {
        owner: Address::repeat_byte(0x42),
        evaluable: EvaluableV4 {
            interpreter: Address::repeat_byte(0xAA),
            store: Address::repeat_byte(0xBB),
            bytecode: Bytes::from(vec![0u8]),
        },
        validInputs: vec![IOV2 {
            token: Address::repeat_byte(0x10),
            vaultId: B256::from([1u8; 32]),
        }],
        validOutputs: vec![IOV2 {
            token: Address::repeat_byte(0x20),
            vaultId: B256::from([2u8; 32]),
        }],
        nonce: B256::ZERO,
    }
}

fn cache_with_code(order: &OrderV4) -> Arc<StaticCodeCache> {
    let cache = Arc::new(StaticCodeCache::default());
    cache.upsert_interpreter(order.evaluable.interpreter, &[0u8]);
    cache.upsert_store(order.evaluable.store, &[0u8]);
    cache
}

fn new_quote_request(order_ref: OrderRef) -> QuoteRequest {
    QuoteRequest::new(order_ref, 0, 0, Address::repeat_byte(0xE1))
}

fn parse_float(value: &str) -> Float {
    Float::parse(value.to_string()).expect("float parse")
}

#[test]
fn snapshot_defaults_to_zeroed_env() {
    let raindex = new_raindex();
    let snapshot = raindex.snapshot();
    assert_eq!(snapshot.env, Env::default());
}

#[test]
fn set_env_updates_fields() {
    let mut raindex = new_raindex();

    raindex
        .apply_mutations(&[RaindexMutation::SetEnv {
            block_number: Some(42),
            timestamp: Some(1337),
        }])
        .expect("set env mutation should succeed");

    assert_eq!(
        raindex.snapshot().env,
        Env {
            block_number: 42,
            timestamp: 1337
        }
    );
}

#[test]
fn batch_recurses_and_preserves_missing_fields() {
    let mut raindex = new_raindex();

    let batch = RaindexMutation::Batch(vec![
        RaindexMutation::SetEnv {
            block_number: Some(1),
            timestamp: None,
        },
        RaindexMutation::Batch(vec![RaindexMutation::SetEnv {
            block_number: None,
            timestamp: Some(2),
        }]),
    ]);

    raindex
        .apply_mutations(&[batch])
        .expect("batch mutation should succeed");

    assert_eq!(
        raindex.snapshot().env,
        Env {
            block_number: 1,
            timestamp: 2
        }
    );
}

#[test]
fn nested_set_orders_prepare_context() {
    let order = test_order();
    let cache = cache_with_code(&order);
    let interpreter = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
    let mut raindex = VirtualRaindex::new(order.owner, cache, interpreter);

    let nested = RaindexMutation::Batch(vec![RaindexMutation::Batch(vec![
        RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        },
    ])]);

    raindex
        .apply_mutations(&[nested])
        .expect("nested set orders should succeed");

    let snapshot = raindex.snapshot();
    let expected_key = VaultKey::new(
        order.owner,
        order.validOutputs[0].token,
        order.validOutputs[0].vaultId,
    );
    assert!(
        snapshot.vault_balances.contains_key(&expected_key),
        "ensure_order_context should seed output vault balance entry"
    );
}

#[test]
fn set_orders_insert_and_remove() {
    let mut raindex = new_raindex();
    let order_a = OrderV4 {
        nonce: B256::from([1u8; 32]),
        ..Default::default()
    };
    let order_b = OrderV4 {
        nonce: B256::from([2u8; 32]),
        ..Default::default()
    };

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order_a.clone(), order_b.clone()],
        }])
        .expect("set orders should succeed");

    let snapshot = raindex.snapshot();
    assert_eq!(snapshot.orders.len(), 2);

    let hash_a = order_hash(&order_a);
    let hash_b = order_hash(&order_b);
    assert!(snapshot.orders.contains_key(&hash_a));
    assert!(snapshot.orders.contains_key(&hash_b));

    raindex
        .apply_mutations(&[RaindexMutation::RemoveOrders {
            order_hashes: vec![hash_a],
        }])
        .expect("remove order should succeed");

    let snapshot = raindex.snapshot();
    assert!(!snapshot.orders.contains_key(&hash_a));
    assert!(snapshot.orders.contains_key(&hash_b));
}

#[test]
fn set_orders_idempotent() {
    let mut raindex = new_raindex();
    let order = OrderV4 {
        nonce: B256::from([7u8; 32]),
        ..Default::default()
    };

    let mutation = RaindexMutation::SetOrders {
        orders: vec![order.clone()],
    };

    raindex
        .apply_mutations(&[mutation.clone()])
        .expect("initial insert should succeed");
    raindex
        .apply_mutations(&[mutation])
        .expect("re-inserting identical order should succeed");

    let snapshot = raindex.snapshot();
    assert_eq!(snapshot.orders.len(), 1);
    let hash = order_hash(&order);
    assert_eq!(snapshot.orders.get(&hash), Some(&order));
}

#[test]
fn vault_delta_accumulates() {
    let mut raindex = new_raindex();
    let owner = Address::repeat_byte(0x01);
    let token = Address::repeat_byte(0x02);
    let vault_id = B256::from([9u8; 32]);

    let add = VaultDelta {
        owner,
        token,
        vault_id,
        delta: parse_float("1"),
    };
    raindex
        .apply_mutations(&[RaindexMutation::VaultDeltas { deltas: vec![add] }])
        .expect("first delta should succeed");

    let sub = VaultDelta {
        owner,
        token,
        vault_id,
        delta: parse_float("-0.4"),
    };
    raindex
        .apply_mutations(&[RaindexMutation::VaultDeltas { deltas: vec![sub] }])
        .expect("second delta should succeed");

    let snapshot = raindex.snapshot();
    let key = VaultKey::new(owner, token, vault_id);
    let balance = snapshot.vault_balances.get(&key).expect("balance entry");
    let expected = (parse_float("1") + parse_float("-0.4")).expect("float math");
    assert_eq!(balance.get_inner(), expected.get_inner());
}

#[test]
fn apply_store_sets_values() {
    let mut raindex = new_raindex();
    let store = Address::repeat_byte(0xaa);
    let fqn = B256::from([3u8; 32]);
    let key = B256::from([4u8; 32]);
    let value = B256::from([5u8; 32]);

    raindex
        .apply_mutations(&[RaindexMutation::ApplyStore {
            sets: vec![StoreSet {
                store,
                fqn,
                kvs: vec![StoreKeyValue { key, value }],
            }],
        }])
        .expect("apply store should succeed");

    let snapshot = raindex.snapshot();
    let store_key = StoreKey::new(store, fqn, key);
    assert_eq!(snapshot.store.get(&store_key), Some(&value));
}

#[test]
fn take_orders_returns_mutations() {
    let order = test_order();
    let cache = cache_with_code(&order);

    let calc_outcome = host::EvalOutcome {
        stack: vec![parse_float("1").get_inner(), parse_float("1").get_inner()],
        writes: vec![
            B256::from(U256::from(7_u64)),
            B256::from(U256::from(11_u64)),
        ],
    };
    let handle_outcome = host::EvalOutcome {
        stack: Vec::new(),
        writes: vec![
            B256::from(U256::from(13_u64)),
            B256::from(U256::from(17_u64)),
        ],
    };

    let host = Arc::new(RecordingHost::new(calc_outcome.clone()));
    host.push_outcome(calc_outcome.clone());
    host.push_outcome(handle_outcome.clone());

    let orderbook = Address::repeat_byte(0xAA);
    let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

    let hash = order_hash(&order);

    raindex
        .apply_mutations(&[RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: order.validInputs[0].token,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: order.validOutputs[0].token,
                    decimals: 18,
                },
            ],
        }])
        .expect("set decimals");

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])
        .expect("set order");

    raindex
        .apply_mutations(&[RaindexMutation::VaultDeltas {
            deltas: vec![VaultDelta {
                owner: order.owner,
                token: order.validOutputs[0].token,
                vault_id: order.validOutputs[0].vaultId,
                delta: parse_float("5"),
            }],
        }])
        .expect("seed vault");

    let config = TakeOrdersConfig {
        orders: vec![TakeOrder {
            order: OrderRef::ByHash(hash),
            input_io_index: 0,
            output_io_index: 0,
            signed_context: Vec::new(),
        }],
        minimum_input: parse_float("0"),
        maximum_input: parse_float("1"),
        maximum_io_ratio: parse_float("10"),
        taker: Address::repeat_byte(0xDD),
        data: Vec::new(),
    };

    let input_key = VaultKey::new(
        order.owner,
        order.validInputs[0].token,
        order.validInputs[0].vaultId,
    );
    let output_key = VaultKey::new(
        order.owner,
        order.validOutputs[0].token,
        order.validOutputs[0].vaultId,
    );

    let before_snapshot = raindex.snapshot();

    let outcome = raindex
        .take_orders(config.clone())
        .expect("simulate take orders");

    assert_eq!(outcome.taken.len(), 1);
    assert!(outcome.warnings.is_empty());
    assert_eq!(
        outcome.total_input.get_inner(),
        parse_float("1").get_inner()
    );
    assert_eq!(
        outcome.total_output.get_inner(),
        parse_float("1").get_inner()
    );
    assert!(!outcome.mutations.is_empty());

    let after_sim_snapshot = raindex.snapshot();
    let before_input = before_snapshot
        .vault_balances
        .get(&input_key)
        .cloned()
        .unwrap_or_default();
    let after_input = after_sim_snapshot
        .vault_balances
        .get(&input_key)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        before_input.format().expect("format before input"),
        after_input.format().expect("format after input")
    );

    let before_output = before_snapshot
        .vault_balances
        .get(&output_key)
        .cloned()
        .unwrap_or_default();
    let after_output = after_sim_snapshot
        .vault_balances
        .get(&output_key)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        before_output.format().expect("format before output"),
        after_output.format().expect("format after output")
    );

    host.push_outcome(calc_outcome.clone());
    host.push_outcome(handle_outcome.clone());

    let applied = raindex
        .take_orders_and_apply_state(config)
        .expect("apply take orders");
    assert_eq!(applied.taken.len(), 1);

    let applied_snapshot = raindex.snapshot();

    let applied_input = applied_snapshot
        .vault_balances
        .get(&input_key)
        .cloned()
        .unwrap_or_default();
    let applied_output = applied_snapshot
        .vault_balances
        .get(&output_key)
        .cloned()
        .unwrap_or_default();

    assert_eq!(
        applied_input.format().expect("format input"),
        parse_float("1").format().expect("expected input format"),
    );
    assert_eq!(
        applied_output.format().expect("format output"),
        parse_float("4").format().expect("expected output format"),
    );

    let qualified = derive_fqn(address_to_u256(order.owner), orderbook);
    let calc_key = StoreKey::new(
        order.evaluable.store,
        qualified,
        B256::from(U256::from(7_u64)),
    );
    let handle_key = StoreKey::new(
        order.evaluable.store,
        qualified,
        B256::from(U256::from(13_u64)),
    );

    assert_eq!(
        applied_snapshot.store.get(&calc_key),
        Some(&B256::from(U256::from(11_u64)))
    );
    assert_eq!(
        applied_snapshot.store.get(&handle_key),
        Some(&B256::from(U256::from(17_u64)))
    );
}

#[test]
fn take_orders_enforces_minimum_input() {
    let order = test_order();
    let cache = cache_with_code(&order);
    let host = Arc::new(RecordingHost::new(host::EvalOutcome {
        stack: vec![parse_float("1").get_inner(), parse_float("0.5").get_inner()],
        writes: Vec::new(),
    }));

    let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    raindex
        .apply_mutations(&[RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: order.validInputs[0].token,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: order.validOutputs[0].token,
                    decimals: 18,
                },
            ],
        }])
        .expect("set decimals");

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])
        .expect("set order");

    raindex
        .apply_mutations(&[RaindexMutation::VaultDeltas {
            deltas: vec![VaultDelta {
                owner: order.owner,
                token: order.validOutputs[0].token,
                vault_id: order.validOutputs[0].vaultId,
                delta: parse_float("1"),
            }],
        }])
        .expect("seed vault");

    let hash = order_hash(&order);
    let err = raindex
        .take_orders(TakeOrdersConfig {
            orders: vec![TakeOrder {
                order: OrderRef::ByHash(hash),
                input_io_index: 0,
                output_io_index: 0,
                signed_context: Vec::new(),
            }],
            minimum_input: parse_float("0.75"),
            maximum_input: parse_float("0.5"),
            maximum_io_ratio: parse_float("10"),
            taker: Address::repeat_byte(0xEE),
            data: Vec::new(),
        })
        .expect_err("minimum input");

    assert_matches::assert_matches!(err, RaindexError::MinimumInputNotMet { .. });
}

#[test]
fn take_orders_skips_ratio_exceeded() {
    let order = test_order();
    let cache = cache_with_code(&order);
    let host = Arc::new(RecordingHost::new(host::EvalOutcome {
        stack: vec![parse_float("5").get_inner(), parse_float("1").get_inner()],
        writes: Vec::new(),
    }));

    let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    raindex
        .apply_mutations(&[RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: order.validInputs[0].token,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: order.validOutputs[0].token,
                    decimals: 18,
                },
            ],
        }])
        .expect("set decimals");

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])
        .expect("set order");

    let hash = order_hash(&order);
    let outcome = raindex
        .take_orders(TakeOrdersConfig {
            orders: vec![TakeOrder {
                order: OrderRef::ByHash(hash),
                input_io_index: 0,
                output_io_index: 0,
                signed_context: Vec::new(),
            }],
            minimum_input: parse_float("0"),
            maximum_input: parse_float("10"),
            maximum_io_ratio: parse_float("1"),
            taker: Address::repeat_byte(0xEF),
            data: Vec::new(),
        })
        .expect("take orders");

    assert!(outcome.taken.is_empty());
    assert_matches::assert_matches!(
        outcome.warnings.first(),
        Some(TakeOrderWarning::RatioExceeded { .. })
    );
}

#[test]
fn set_token_decimals() {
    let mut raindex = new_raindex();
    let token_a = Address::repeat_byte(0xa1);
    let token_b = Address::repeat_byte(0xb2);

    raindex
        .apply_mutations(&[RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: token_a,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: token_b,
                    decimals: 6,
                },
            ],
        }])
        .expect("set token decimals should succeed");

    let snapshot = raindex.snapshot();
    assert_eq!(snapshot.token_decimals.get(&token_a), Some(&18));
    assert_eq!(snapshot.token_decimals.get(&token_b), Some(&6));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn revm_host_matches_contract() {
    let local_evm = LocalEvm::new().await;
    let rain_src = b"/* 0. calculate-io */\n_ _: 10 20;\n\n/* 1. handle-io */\n:;".to_vec();

    let parse_return = local_evm
        .deployer
        .parse2(Bytes::from(rain_src.clone()))
        .call()
        .await
        .expect("parse2");

    let order = OrderV4 {
        owner: Address::ZERO,
        evaluable: EvaluableV4 {
            interpreter: *local_evm.interpreter.address(),
            store: *local_evm.store.address(),
            bytecode: parse_return.clone(),
        },
        validInputs: Vec::new(),
        validOutputs: Vec::new(),
        nonce: B256::ZERO,
    };

    let contract_eval = Interpreter::EvalV4 {
        store: order.evaluable.store,
        namespace: U256::ZERO,
        bytecode: parse_return.clone(),
        sourceIndex: U256::ZERO,
        context: vec![],
        inputs: vec![],
        stateOverlay: vec![],
    };

    let expected = local_evm
        .interpreter
        .eval4(contract_eval.clone())
        .call()
        .await
        .expect("contract eval");
    let (expected_stack, expected_writes) = (expected._0, expected._1);

    let cache = Arc::new(StaticCodeCache::default());
    cache.upsert_interpreter(
        order.evaluable.interpreter,
        Interpreter::DEPLOYED_BYTECODE.as_ref(),
    );
    cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

    let host = RevmInterpreterHost::new(cache);

    let eval = EvalV4 {
        store: order.evaluable.store,
        namespace: U256::ZERO,
        bytecode: order.evaluable.bytecode.clone(),
        sourceIndex: U256::ZERO,
        context: vec![],
        inputs: vec![],
        stateOverlay: vec![],
    };

    let outcome = host
        .eval4(
            order.evaluable.interpreter,
            &eval,
            &std::collections::HashMap::new(),
            Env::default(),
        )
        .expect("eval4");

    assert_eq!(outcome.stack, expected_stack);
    assert_eq!(outcome.writes, expected_writes);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn quote_matches_contract_eval() {
    let local_evm = LocalEvm::new().await;
    let rain_src = b"/* 0. calculate-io */\n_ _: 10 20;\n\n/* 1. handle-io */\n:;".to_vec();

    let parse_return = local_evm
        .deployer
        .parse2(Bytes::from(rain_src.clone()))
        .call()
        .await
        .expect("parse2");
    let orderbook = Address::repeat_byte(0xAB);
    let input_token = Address::repeat_byte(0x11);
    let output_token = Address::repeat_byte(0x22);
    let input_vault_id = B256::from([1u8; 32]);
    let output_vault_id = B256::from([2u8; 32]);

    let order = OrderV4 {
        owner: Address::repeat_byte(0x42),
        evaluable: EvaluableV4 {
            interpreter: *local_evm.interpreter.address(),
            store: *local_evm.store.address(),
            bytecode: parse_return.clone(),
        },
        validInputs: vec![IOV2 {
            token: input_token,
            vaultId: input_vault_id,
        }],
        validOutputs: vec![IOV2 {
            token: output_token,
            vaultId: output_vault_id,
        }],
        nonce: B256::ZERO,
    };

    let cache = Arc::new(StaticCodeCache::default());
    cache.upsert_interpreter(
        order.evaluable.interpreter,
        Interpreter::DEPLOYED_BYTECODE.as_ref(),
    );
    cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

    let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
    let mut raindex = VirtualRaindex::new(orderbook, cache, host);

    let decimals_mutation = RaindexMutation::SetTokenDecimals {
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
    };

    let output_balance = parse_float("5");
    let vault_delta = RaindexMutation::VaultDeltas {
        deltas: vec![
            VaultDelta {
                owner: order.owner,
                token: input_token,
                vault_id: input_vault_id,
                delta: Float::default(),
            },
            VaultDelta {
                owner: order.owner,
                token: output_token,
                vault_id: output_vault_id,
                delta: output_balance,
            },
        ],
    };

    raindex
        .apply_mutations(&[
            decimals_mutation,
            RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            },
            vault_delta,
        ])
        .expect("mutations");

    let order_hash = order_hash(&order);
    let state_namespace = address_to_u256(order.owner);
    let fqn = derive_fqn(state_namespace, orderbook);
    let namespace = U256::from_be_slice(fqn.as_slice());

    let counterparty = Address::repeat_byte(0xE1);
    let input_balance = Float::default();
    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        &IOContext {
            io: order.validInputs[0].clone(),
            balance: input_balance,
            decimals: 18,
        },
        &IOContext {
            io: order.validOutputs[0].clone(),
            balance: output_balance,
            decimals: 18,
        },
        &[],
    );

    let contract_eval = Interpreter::EvalV4 {
        store: order.evaluable.store,
        namespace,
        bytecode: parse_return.clone(),
        sourceIndex: U256::ZERO,
        context: context.clone(),
        inputs: vec![],
        stateOverlay: vec![],
    };

    let expected = local_evm
        .interpreter
        .eval4(contract_eval)
        .call()
        .await
        .expect("contract eval");
    let expected_stack = expected._0;
    assert_eq!(expected_stack.len(), 2);

    let stored_quote = raindex
        .quote(QuoteRequest::new(
            OrderRef::ByHash(order_hash),
            0,
            0,
            counterparty,
        ))
        .expect("stored quote");

    let inline_quote = raindex
        .quote(QuoteRequest::new(
            OrderRef::Inline(order.clone()),
            0,
            0,
            counterparty,
        ))
        .expect("inline quote");

    let expected_ratio = Float::from_raw(expected_stack[0]);
    let expected_max = Float::from_raw(expected_stack[1])
        .min(output_balance)
        .expect("min");

    assert_eq!(
        stored_quote.io_ratio.get_inner(),
        expected_ratio.get_inner()
    );
    assert_eq!(
        stored_quote.output_max.get_inner(),
        expected_max.get_inner()
    );
    assert_eq!(
        inline_quote.io_ratio.get_inner(),
        expected_ratio.get_inner()
    );
    assert_eq!(
        inline_quote.output_max.get_inner(),
        expected_max.get_inner()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn quote_reflects_env_values() {
    let local_evm = LocalEvm::new().await;
    let rain_src =
        b"/* 0. calculate-io */\n_ _: now() block-number();\n\n/* 1. handle-io */\n:;".to_vec();

    let parse_return = local_evm
        .deployer
        .parse2(Bytes::from(rain_src.clone()))
        .call()
        .await
        .expect("parse2");

    let orderbook = Address::repeat_byte(0xCC);
    let input_token = Address::repeat_byte(0x11);
    let output_token = Address::repeat_byte(0x22);
    let input_vault_id = B256::from([0xA1; 32]);
    let output_vault_id = B256::from([0xB2; 32]);

    let order = OrderV4 {
        owner: Address::repeat_byte(0x42),
        evaluable: EvaluableV4 {
            interpreter: *local_evm.interpreter.address(),
            store: *local_evm.store.address(),
            bytecode: parse_return.clone(),
        },
        validInputs: vec![IOV2 {
            token: input_token,
            vaultId: input_vault_id,
        }],
        validOutputs: vec![IOV2 {
            token: output_token,
            vaultId: output_vault_id,
        }],
        nonce: B256::ZERO,
    };

    let cache = Arc::new(StaticCodeCache::default());
    cache.upsert_interpreter(
        order.evaluable.interpreter,
        Interpreter::DEPLOYED_BYTECODE.as_ref(),
    );
    cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

    let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
    let mut raindex = VirtualRaindex::new(orderbook, cache, host);

    let token_decimals = RaindexMutation::SetTokenDecimals {
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
    };

    let initial_balance = parse_float("1000000");
    let vault_delta = RaindexMutation::VaultDeltas {
        deltas: vec![
            VaultDelta {
                owner: order.owner,
                token: input_token,
                vault_id: input_vault_id,
                delta: Float::default(),
            },
            VaultDelta {
                owner: order.owner,
                token: output_token,
                vault_id: output_vault_id,
                delta: initial_balance,
            },
        ],
    };

    let env_first = Env {
        block_number: 1_234,
        timestamp: 5_678,
    };

    let env_mutation = RaindexMutation::SetEnv {
        block_number: Some(env_first.block_number),
        timestamp: Some(env_first.timestamp),
    };

    let order_mutation = RaindexMutation::SetOrders {
        orders: vec![order.clone()],
    };

    raindex
        .apply_mutations(&[
            token_decimals,
            vault_delta.clone(),
            env_mutation,
            order_mutation,
        ])
        .expect("initial mutations");

    let order_hash = order_hash(&order);
    let request = QuoteRequest::new(OrderRef::ByHash(order_hash), 0, 0, Address::ZERO);

    let quote_first = raindex.quote(request.clone()).expect("quote env 1");
    // Rainlang pushes `block-number()` then `now()` so stack[0] maps to the block.
    let first_block = quote_first
        .io_ratio
        .to_fixed_decimal(0)
        .expect("block to fixed");
    assert_eq!(first_block, U256::from(env_first.block_number));

    let first_timestamp = quote_first
        .output_max
        .to_fixed_decimal(0)
        .expect("timestamp to fixed");
    assert_eq!(first_timestamp, U256::from(env_first.timestamp));

    let env_second = Env {
        block_number: 9_999,
        timestamp: 44_444,
    };

    raindex
        .apply_mutations(&[RaindexMutation::SetEnv {
            block_number: Some(env_second.block_number),
            timestamp: Some(env_second.timestamp),
        }])
        .expect("update env");

    // refresh vault balance so the min check does not clip our output
    raindex
        .apply_mutations(&[vault_delta.clone()])
        .expect("refresh balance");

    let quote_second = raindex.quote(request).expect("quote env 2");
    let second_block = quote_second
        .io_ratio
        .to_fixed_decimal(0)
        .expect("block to fixed second");
    assert_eq!(second_block, U256::from(env_second.block_number));

    let second_timestamp = quote_second
        .output_max
        .to_fixed_decimal(0)
        .expect("timestamp to fixed second");
    assert_eq!(second_timestamp, U256::from(env_second.timestamp));
}

#[test]
fn quote_errors_when_order_missing() {
    let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
    let cache = Arc::new(StaticCodeCache::default());
    let raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    let err = raindex
        .quote(new_quote_request(OrderRef::ByHash(B256::ZERO)))
        .expect_err("missing order");
    assert_matches::assert_matches!(err, RaindexError::OrderNotFound { .. });
}

#[test]
fn quote_errors_on_invalid_io_index() {
    let mut order = test_order();
    order.validInputs.push(IOV2 {
        token: Address::repeat_byte(0x11),
        vaultId: B256::from([3u8; 32]),
    });

    let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
    let cache = cache_with_code(&order);
    let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])
        .expect("set order");

    let err = raindex
        .quote(QuoteRequest::new(
            OrderRef::ByHash(order_hash(&order)),
            2,
            0,
            Address::ZERO,
        ))
        .expect_err("invalid input index");
    assert_matches::assert_matches!(err, RaindexError::InvalidInputIndex { .. });

    let err = raindex
        .quote(QuoteRequest::new(
            OrderRef::ByHash(order_hash(&order)),
            0,
            1,
            Address::ZERO,
        ))
        .expect_err("invalid output index");
    assert_matches::assert_matches!(err, RaindexError::InvalidOutputIndex { .. });
}

#[test]
fn quote_errors_without_token_decimals() {
    let order = test_order();
    let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
    let cache = cache_with_code(&order);
    let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    raindex
        .apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])
        .expect("set order");

    let err = raindex
        .quote(new_quote_request(OrderRef::ByHash(order_hash(&order))))
        .expect_err("missing decimals");
    assert_matches::assert_matches!(err, RaindexError::TokenDecimalMissing { .. });
}

#[test]
fn add_order_runs_post_tasks() {
    let order = test_order();
    let cache = cache_with_code(&order);

    let key = B256::from(U256::from(7_u64));
    let value = B256::from(U256::from(42_u64));
    let outcome = host::EvalOutcome {
        stack: Vec::new(),
        writes: vec![key, value],
    };

    let host = Arc::new(RecordingHost::new(outcome));
    let orderbook = Address::repeat_byte(0x99);
    let mut raindex = VirtualRaindex::new(orderbook, cache, host);

    let task = TaskV2 {
        evaluable: order.evaluable.clone(),
        signedContext: Vec::new(),
    };

    raindex
        .add_order(order.clone(), vec![task])
        .expect("add order succeeds");

    let namespace = address_to_u256(order.owner);
    let qualified = derive_fqn(namespace, raindex.orderbook_address());
    let store_key = StoreKey::new(order.evaluable.store, qualified, key);

    let snapshot = raindex.snapshot();
    assert_eq!(snapshot.store.get(&store_key), Some(&value));
}

#[test]
fn quote_errors_on_self_trade() {
    let mut order = test_order();
    order.validOutputs[0].token = order.validInputs[0].token;

    let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
    let cache = cache_with_code(&order);
    let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

    raindex
        .apply_mutations(&[
            RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            },
            RaindexMutation::SetTokenDecimals {
                entries: vec![TokenDecimalEntry {
                    token: order.validInputs[0].token,
                    decimals: 18,
                }],
            },
        ])
        .expect("mutations");

    let err = raindex
        .quote(new_quote_request(OrderRef::ByHash(order_hash(&order))))
        .expect_err("self trade");
    assert_matches::assert_matches!(err, RaindexError::TokenSelfTrade);
}

#[test]
fn quote_builds_expected_eval_context() {
    let order = test_order();
    let order_hash = order_hash(&order);
    let output_balance = parse_float("5");

    let write_key = B256::from([0xAA; 32]);
    let write_value = B256::from([0xBB; 32]);
    let outcome = host::EvalOutcome {
        stack: vec![
            parse_float("0.5").get_inner(),
            parse_float("10").get_inner(),
        ],
        writes: vec![write_key, write_value],
    };
    let host = Arc::new(RecordingHost::new(outcome));
    let cache = cache_with_code(&order);
    let orderbook = Address::repeat_byte(0xAB);
    let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

    raindex
        .apply_mutations(&[
            RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: order.validOutputs[0].token,
                        decimals: 18,
                    },
                ],
            },
            RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            },
            RaindexMutation::VaultDeltas {
                deltas: vec![
                    VaultDelta {
                        owner: order.owner,
                        token: order.validInputs[0].token,
                        vault_id: order.validInputs[0].vaultId,
                        delta: Float::default(),
                    },
                    VaultDelta {
                        owner: order.owner,
                        token: order.validOutputs[0].token,
                        vault_id: order.validOutputs[0].vaultId,
                        delta: output_balance,
                    },
                ],
            },
        ])
        .expect("mutations");

    let quote = raindex
        .quote(new_quote_request(OrderRef::ByHash(order_hash)))
        .expect("quote");

    assert_eq!(quote.writes, vec![write_key, write_value]);
    assert_eq!(quote.io_ratio.get_inner(), parse_float("0.5").get_inner());
    assert_eq!(quote.output_max.get_inner(), output_balance.get_inner());

    let recorded = host.last_eval().expect("eval recorded");
    assert_eq!(recorded.sourceIndex, U256::ZERO);

    let namespace = derive_fqn(address_to_u256(order.owner), orderbook);
    assert_eq!(
        recorded.namespace,
        U256::from_be_slice(namespace.as_slice())
    );

    assert_eq!(recorded.context.len(), 5);
    let base_column = &recorded.context[0];
    assert_eq!(base_column[0], Address::repeat_byte(0xE1).into_word());
    assert_eq!(base_column[1], orderbook.into_word());

    let calling_context = &recorded.context[CONTEXT_CALLING_CONTEXT_COLUMN];
    assert_eq!(calling_context.len(), 3);
    assert_eq!(calling_context[0], order_hash);
    assert_eq!(calling_context[1], order.owner.into_word());

    let vault_inputs = &recorded.context[CONTEXT_VAULT_INPUTS_COLUMN];
    assert_eq!(vault_inputs[0], order.validInputs[0].token.into_word());
    assert_eq!(vault_inputs[3], Float::default().get_inner());

    let vault_outputs = &recorded.context[CONTEXT_VAULT_OUTPUTS_COLUMN];
    assert_eq!(vault_outputs[3], output_balance.get_inner());
}

#[test]
fn quote_applies_store_overrides() {
    let order = test_order();
    let order_hash = order_hash(&order);

    let outcome = host::EvalOutcome {
        stack: vec![parse_float("1").get_inner(), parse_float("1").get_inner()],
        writes: Vec::new(),
    };
    let host = Arc::new(RecordingHost::new(outcome));
    let cache = cache_with_code(&order);
    let orderbook = Address::repeat_byte(0xAB);
    let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

    let namespace = derive_fqn(address_to_u256(order.owner), orderbook);
    let existing_key = B256::from([0xA5; 32]);
    let existing_value = B256::from([0xB6; 32]);
    let input_balance = Float::default();
    let output_balance = parse_float("1");

    raindex
        .apply_mutations(&[
            RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: order.validOutputs[0].token,
                        decimals: 18,
                    },
                ],
            },
            RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            },
            RaindexMutation::VaultDeltas {
                deltas: vec![
                    VaultDelta {
                        owner: order.owner,
                        token: order.validInputs[0].token,
                        vault_id: order.validInputs[0].vaultId,
                        delta: input_balance,
                    },
                    VaultDelta {
                        owner: order.owner,
                        token: order.validOutputs[0].token,
                        vault_id: order.validOutputs[0].vaultId,
                        delta: output_balance,
                    },
                ],
            },
            RaindexMutation::ApplyStore {
                sets: vec![StoreSet {
                    store: order.evaluable.store,
                    fqn: namespace,
                    kvs: vec![StoreKeyValue {
                        key: existing_key,
                        value: existing_value,
                    }],
                }],
            },
        ])
        .expect("mutations");

    let override_entry = StoreOverride {
        store: order.evaluable.store,
        fqn: namespace,
        key: B256::from([0xC7; 32]),
        value: B256::from([0xD8; 32]),
    };

    raindex
        .quote(new_quote_request(OrderRef::ByHash(order_hash)).with_overrides(vec![override_entry]))
        .expect("quote");

    let snapshot = host.last_snapshot().expect("snapshot recorded");
    assert_eq!(snapshot.len(), 2);
    assert_eq!(
        snapshot
            .get(&StoreKey::new(
                order.evaluable.store,
                namespace,
                existing_key
            ))
            .copied()
            .unwrap(),
        existing_value
    );
    assert_eq!(
        snapshot
            .get(&StoreKey::new(
                order.evaluable.store,
                namespace,
                override_entry.key
            ))
            .copied()
            .unwrap(),
        override_entry.value
    );
}
