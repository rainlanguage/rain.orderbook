//! Virtual Raindex state representation and mutation helpers.

use std::collections::HashMap;

use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol_types::SolValue,
};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;
use rain_orderbook_common::utils::order_hash;

/// Shared environmental information for the Virtual Raindex.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Env {
    pub block_number: u64,
    pub timestamp: u64,
}

/// Key that uniquely identifies a vault balance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VaultKey {
    pub owner: Address,
    pub token: Address,
    pub vault_id: B256,
}

impl VaultKey {
    pub const fn new(owner: Address, token: Address, vault_id: B256) -> Self {
        Self {
            owner,
            token,
            vault_id,
        }
    }
}

/// Key that uniquely identifies a store value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoreKey {
    pub store: Address,
    pub fqn: B256,
    pub key: B256,
}

impl StoreKey {
    pub const fn new(store: Address, fqn: B256, key: B256) -> Self {
        Self { store, fqn, key }
    }
}

/// Serialized representation of the Virtual Raindex state.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub env: Env,
    pub orders: HashMap<B256, OrderV4>,
    pub vault_balances: HashMap<VaultKey, Float>,
    pub store: HashMap<StoreKey, B256>,
    pub token_decimals: HashMap<Address, u8>,
}

/// Mutation payloads that describe how to update Virtual Raindex state.
#[derive(Clone, Debug)]
pub enum RaindexMutation {
    /// Update the environment values. `None` values leave the existing field untouched.
    SetEnv {
        block_number: Option<u64>,
        timestamp: Option<u64>,
    },
    /// Insert orders indexed by their canonical hash. Re-adding the same order is
    /// idempotent because the hash is derived from the full order payload.
    SetOrders { orders: Vec<OrderV4> },
    /// Remove specific orders by hash.
    RemoveOrders { order_hashes: Vec<B256> },
    /// Apply deltas to vault balances.
    VaultDeltas { deltas: Vec<VaultDelta> },
    /// Apply key/value writes to interpreter store namespaces.
    ApplyStore { sets: Vec<StoreSet> },
    /// Record token decimals.
    SetTokenDecimals { entries: Vec<TokenDecimalEntry> },
    /// Apply multiple mutations atomically.
    Batch(Vec<RaindexMutation>),
}

/// Vault delta descriptor used in [RaindexMutation::VaultDeltas].
#[derive(Clone, Debug)]
pub struct VaultDelta {
    pub owner: Address,
    pub token: Address,
    pub vault_id: B256,
    pub delta: Float,
}

/// Store write descriptor used in [RaindexMutation::ApplyStore].
#[derive(Clone, Debug)]
pub struct StoreSet {
    pub store: Address,
    pub fqn: B256,
    pub kvs: Vec<StoreKeyValue>,
}

/// Store key/value pair.
#[derive(Clone, Debug)]
pub struct StoreKeyValue {
    pub key: B256,
    pub value: B256,
}

/// Token decimal entry used in [RaindexMutation::SetTokenDecimals].
#[derive(Clone, Debug)]
pub struct TokenDecimalEntry {
    pub token: Address,
    pub decimals: u8,
}

/// In-memory state backing a [`VirtualRaindex`](crate::VirtualRaindex) instance.
#[derive(Clone, Debug, Default)]
pub struct RaindexState {
    pub env: Env,
    pub orders: HashMap<B256, OrderV4>,
    pub vault_balances: HashMap<VaultKey, Float>,
    pub store: HashMap<StoreKey, B256>,
    pub token_decimals: HashMap<Address, u8>,
}

impl RaindexState {
    /// Captures a full copy of the current state for inspection or serialization.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            env: self.env,
            orders: self.orders.clone(),
            vault_balances: self.vault_balances.clone(),
            store: self.store.clone(),
            token_decimals: self.token_decimals.clone(),
        }
    }

    /// Applies a batch of mutations, recursing through nested batches as needed.
    pub fn apply_mutations(&mut self, mutations: &[RaindexMutation]) -> crate::Result<()> {
        for mutation in mutations {
            match mutation {
                RaindexMutation::SetEnv {
                    block_number,
                    timestamp,
                } => {
                    if let Some(block_number) = block_number {
                        self.env.block_number = *block_number;
                    }
                    if let Some(timestamp) = timestamp {
                        self.env.timestamp = *timestamp;
                    }
                }
                RaindexMutation::SetOrders { orders } => {
                    for order in orders {
                        let hash = order_hash(order);
                        self.orders.insert(hash, order.clone());
                    }
                }
                RaindexMutation::RemoveOrders { order_hashes } => {
                    for hash in order_hashes {
                        self.orders.remove(hash);
                    }
                }
                RaindexMutation::VaultDeltas { deltas } => {
                    for delta in deltas {
                        self.apply_vault_delta(delta)?;
                    }
                }
                RaindexMutation::ApplyStore { sets } => {
                    for set in sets {
                        for kv in &set.kvs {
                            let key = StoreKey::new(set.store, set.fqn, kv.key);
                            self.store.insert(key, kv.value);
                        }
                    }
                }
                RaindexMutation::SetTokenDecimals { entries } => {
                    for entry in entries {
                        self.token_decimals.insert(entry.token, entry.decimals);
                    }
                }
                RaindexMutation::Batch(batch) => self.apply_mutations(batch)?,
            }
        }

        Ok(())
    }

    /// Applies a single vault balance delta to the state.
    fn apply_vault_delta(&mut self, delta: &VaultDelta) -> crate::Result<()> {
        let key = VaultKey::new(delta.owner, delta.token, delta.vault_id);
        let entry = self.vault_balances.entry(key).or_default();
        let updated = (*entry + delta.delta)?;
        *entry = updated;
        Ok(())
    }
}

/// Derives the fully-qualified namespace for a Rain interpreter store namespace.
pub fn derive_fqn(namespace: U256, caller: Address) -> B256 {
    keccak256((namespace, caller).abi_encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, Bytes, U256};
    use alloy::sol_types::SolValue;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, IOV2};

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_string()).expect("float parse")
    }

    fn sample_order(owner_byte: u8, nonce_byte: u8) -> OrderV4 {
        let mut order = OrderV4::default();
        order.owner = Address::repeat_byte(owner_byte);
        order.nonce = B256::from([nonce_byte; 32]);
        order.evaluable = EvaluableV4 {
            interpreter: Address::repeat_byte(0xF1),
            store: Address::repeat_byte(0xF2),
            bytecode: Bytes::default(),
        };
        order.validInputs = vec![IOV2 {
            token: Address::repeat_byte(0x11),
            vaultId: B256::from([0xAA; 32]),
        }];
        order.validOutputs = vec![IOV2 {
            token: Address::repeat_byte(0x22),
            vaultId: B256::from([0xBB; 32]),
        }];
        order
    }

    #[test]
    fn apply_mutations_updates_all_state_segments() {
        let mut state = RaindexState::default();
        let order_a = sample_order(0xAA, 0x01);
        let order_b = sample_order(0xBB, 0x02);
        let hash_a = order_hash(&order_a);
        let hash_b = order_hash(&order_b);

        let vault_key = VaultKey::new(
            Address::repeat_byte(0x0C),
            Address::repeat_byte(0x0D),
            B256::from([0xCC; 32]),
        );
        let vault_delta = VaultDelta {
            owner: vault_key.owner,
            token: vault_key.token,
            vault_id: vault_key.vault_id,
            delta: parse_float("3.5"),
        };

        let store_key = B256::from([0xDD; 32]);
        let store_value = B256::from([0xEE; 32]);

        state
            .apply_mutations(&[
                RaindexMutation::SetEnv {
                    block_number: Some(123),
                    timestamp: Some(456),
                },
                RaindexMutation::SetOrders {
                    orders: vec![order_a.clone(), order_b.clone()],
                },
                RaindexMutation::Batch(vec![RaindexMutation::VaultDeltas {
                    deltas: vec![vault_delta.clone()],
                }]),
                RaindexMutation::ApplyStore {
                    sets: vec![StoreSet {
                        store: Address::repeat_byte(0x99),
                        fqn: B256::from([0xFE; 32]),
                        kvs: vec![StoreKeyValue {
                            key: store_key,
                            value: store_value,
                        }],
                    }],
                },
                RaindexMutation::SetTokenDecimals {
                    entries: vec![TokenDecimalEntry {
                        token: Address::repeat_byte(0x55),
                        decimals: 6,
                    }],
                },
                RaindexMutation::RemoveOrders {
                    order_hashes: vec![hash_b],
                },
            ])
            .expect("mutations apply");

        assert_eq!(
            state.env,
            Env {
                block_number: 123,
                timestamp: 456
            }
        );
        assert_eq!(state.orders.get(&hash_a), Some(&order_a));
        assert!(!state.orders.contains_key(&hash_b));
        let applied_balance = *state
            .vault_balances
            .get(&vault_key)
            .expect("vault delta balance recorded");
        assert!(applied_balance
            .eq(vault_delta.delta)
            .expect("float eq for applied balance"));
        let store_entry = StoreKey::new(
            Address::repeat_byte(0x99),
            B256::from([0xFE; 32]),
            store_key,
        );
        assert_eq!(state.store.get(&store_entry), Some(&store_value));
        assert_eq!(
            state.token_decimals.get(&Address::repeat_byte(0x55)),
            Some(&6)
        );
    }

    #[test]
    fn apply_vault_delta_creates_new_balance() {
        let mut state = RaindexState::default();
        let delta = VaultDelta {
            owner: Address::repeat_byte(0x01),
            token: Address::repeat_byte(0x02),
            vault_id: B256::from([0x03; 32]),
            delta: parse_float("1.25"),
        };

        state
            .apply_vault_delta(&delta)
            .expect("vault delta applied");

        let key = VaultKey::new(delta.owner, delta.token, delta.vault_id);
        let recorded_balance = *state
            .vault_balances
            .get(&key)
            .expect("vault balance created");
        assert!(recorded_balance
            .eq(delta.delta)
            .expect("float eq for new balance"));
    }

    #[test]
    fn apply_vault_delta_accumulates_existing_balance() {
        let mut state = RaindexState::default();
        let key = VaultKey::new(
            Address::repeat_byte(0x10),
            Address::repeat_byte(0x20),
            B256::from([0x30; 32]),
        );
        state.vault_balances.insert(key, parse_float("4.0"));

        let delta = VaultDelta {
            owner: key.owner,
            token: key.token,
            vault_id: key.vault_id,
            delta: parse_float("1.5"),
        };

        state
            .apply_vault_delta(&delta)
            .expect("vault delta accumulated");

        let updated_balance = *state
            .vault_balances
            .get(&key)
            .expect("vault balance updated");
        assert!(updated_balance
            .eq(parse_float("5.5"))
            .expect("float eq for accumulated balance"));
    }

    #[test]
    fn derive_fqn_matches_expected_hash() {
        let namespace = U256::from(42);
        let caller = Address::repeat_byte(0xCA);
        let expected = keccak256((namespace, caller).abi_encode());
        assert_eq!(derive_fqn(namespace, caller), expected);
    }
}
