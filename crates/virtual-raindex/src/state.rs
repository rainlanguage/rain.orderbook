//! Virtual Raindex state representation and mutation helpers.

use std::collections::HashMap;

use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol_types::SolValue,
};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

use crate::Result;

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
    pub fn apply_mutations(&mut self, mutations: &[RaindexMutation]) -> Result<()> {
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
    fn apply_vault_delta(&mut self, delta: &VaultDelta) -> Result<()> {
        let key = VaultKey::new(delta.owner, delta.token, delta.vault_id);
        let entry = self.vault_balances.entry(key).or_default();
        let updated = (*entry + delta.delta)?;
        *entry = updated;
        Ok(())
    }
}

/// Computes the canonical hash for an [`OrderV4`].
pub fn order_hash(order: &OrderV4) -> B256 {
    keccak256(order.abi_encode())
}

/// Derives the fully-qualified namespace for a Rain interpreter store namespace.
pub fn derive_fqn(namespace: U256, caller: Address) -> B256 {
    keccak256((namespace, caller).abi_encode())
}
