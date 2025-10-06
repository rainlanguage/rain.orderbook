use std::collections::{HashMap, HashSet};

use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};

use crate::{
    state::{Env, Snapshot, StoreKey, TokenDecimalEntry, VaultKey},
    Float,
};
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Human-serializable snapshot representation with explicit cache handles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotBundle {
    pub orderbook: Address,
    pub env: Env,
    pub orders: Vec<OrderRecord>,
    pub vault_balances: Vec<VaultBalanceRecord>,
    pub store: Vec<StoreRecord>,
    pub token_decimals: Vec<TokenDecimalEntry>,
    pub cache_handles: CacheHandles,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderRecord {
    pub hash: B256,
    pub order: OrderV4,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultBalanceRecord {
    pub owner: Address,
    pub token: Address,
    pub vault_id: B256,
    pub balance: Float,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreRecord {
    pub store: Address,
    pub fqn: B256,
    pub key: B256,
    pub value: B256,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CacheHandles {
    pub interpreters: Vec<Address>,
    pub stores: Vec<Address>,
}

impl SnapshotBundle {
    /// Builds a bundle by moving data out of a raw snapshot.
    pub fn from_snapshot(orderbook: Address, snapshot: Snapshot) -> Self {
        let Snapshot {
            env,
            orders,
            vault_balances,
            store,
            token_decimals,
        } = snapshot;

        let orders: Vec<OrderRecord> = orders
            .into_iter()
            .map(|(hash, order)| OrderRecord { hash, order })
            .collect();

        let mut interpreter_handles: HashSet<Address> = HashSet::new();
        let mut store_handles: HashSet<Address> = HashSet::new();
        for record in &orders {
            interpreter_handles.insert(record.order.evaluable.interpreter);
            store_handles.insert(record.order.evaluable.store);
        }

        let mut cache_handles = CacheHandles {
            interpreters: interpreter_handles.into_iter().collect(),
            stores: store_handles.into_iter().collect(),
        };

        cache_handles.interpreters.sort_unstable();
        cache_handles.stores.sort_unstable();

        let vault_balances = vault_balances
            .into_iter()
            .map(
                |(
                    VaultKey {
                        owner,
                        token,
                        vault_id,
                    },
                    balance,
                )| VaultBalanceRecord {
                    owner,
                    token,
                    vault_id,
                    balance,
                },
            )
            .collect();

        let store = store
            .into_iter()
            .map(|(StoreKey { store, fqn, key }, value)| StoreRecord {
                store,
                fqn,
                key,
                value,
            })
            .collect();

        let token_decimals = token_decimals
            .into_iter()
            .map(|(token, decimals)| TokenDecimalEntry { token, decimals })
            .collect();

        Self {
            orderbook,
            env,
            orders,
            vault_balances,
            store,
            token_decimals,
            cache_handles,
        }
    }

    /// Converts the bundle back into a snapshot suitable for state hydration.
    pub fn into_snapshot(self) -> Snapshot {
        let mut orders: HashMap<B256, OrderV4> = HashMap::with_capacity(self.orders.len());
        for record in self.orders {
            orders.insert(record.hash, record.order);
        }

        let mut vault_balances = HashMap::with_capacity(self.vault_balances.len());
        for entry in self.vault_balances {
            vault_balances.insert(
                VaultKey::new(entry.owner, entry.token, entry.vault_id),
                entry.balance,
            );
        }

        let mut store = HashMap::with_capacity(self.store.len());
        for entry in self.store {
            store.insert(
                StoreKey::new(entry.store, entry.fqn, entry.key),
                entry.value,
            );
        }

        let mut token_decimals = HashMap::with_capacity(self.token_decimals.len());
        for entry in self.token_decimals {
            token_decimals.insert(entry.token, entry.decimals);
        }

        Snapshot {
            env: self.env,
            orders,
            vault_balances,
            store,
            token_decimals,
        }
    }

    /// Provides the list of interpreter/store addresses referenced by this snapshot.
    pub fn cache_handles(&self) -> &CacheHandles {
        &self.cache_handles
    }
}
