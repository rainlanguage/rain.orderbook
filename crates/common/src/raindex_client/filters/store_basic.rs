use super::{vaults_builder::VaultsFilterBuilder, vaults_filter::GetVaultsFilters};
use crate::raindex_client::filters::traits::{FilterBuilder, FilterStore};
use crate::raindex_client::*;
use serde::{Deserialize, Serialize};

//
// Basic store for filters
//

#[derive(Serialize, Deserialize, Debug, Clone, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BasicFilterStore {
    pub vaults: GetVaultsFilters,
}

impl BasicFilterStore {
    pub fn new() -> Self {
        Self {
            vaults: GetVaultsFilters::default(),
        }
    }
}

impl FilterStore for BasicFilterStore {
    fn set_vaults_filters(&mut self, filters: GetVaultsFilters) {
        self.vaults = filters;
    }
    fn update_vaults_filters<F>(&mut self, update_fn: F)
    where
        F: FnOnce(VaultsFilterBuilder) -> VaultsFilterBuilder,
    {
        let builder = VaultsFilterBuilder::from(self.vaults.clone());
        let updated_filter = update_fn(builder).build();
        self.vaults = updated_filter;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::filters::traits::{Filter, FilterStore};
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn test_basic_filter_store_new() {
        let store = BasicFilterStore::new();
        assert!(store.vaults.owners.is_empty());
        assert!(!store.vaults.hide_zero_balance);
        assert!(store.vaults.tokens.is_none());
    }

    #[test]
    fn test_basic_filter_store_default() {
        let store = BasicFilterStore::default();
        assert!(store.vaults.owners.is_empty());
        assert!(!store.vaults.hide_zero_balance);
        assert!(store.vaults.tokens.is_none());
    }

    #[test]
    fn test_set_vaults_filters() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let mut store = BasicFilterStore::new();
        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
        };

        store.set_vaults_filters(filters.clone());

        assert_eq!(store.vaults.owners, filters.owners);
        assert_eq!(store.vaults.hide_zero_balance, filters.hide_zero_balance);
        assert_eq!(store.vaults.tokens, filters.tokens);
    }

    #[test]
    fn test_update_vaults_filters_basic() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        let mut store = BasicFilterStore::new();

        // Update Stored value using closure
        store.update_vaults_filters(|builder| {
            builder.set_owners(vec![owner1]).set_hide_zero_balance(true)
        });

        assert_eq!(store.vaults.owners.len(), 1);
        assert_eq!(store.vaults.owners[0], owner1);
        assert!(store.vaults.hide_zero_balance);
        assert!(store.vaults.tokens.is_none());
    }

    #[test]
    fn test_update_vaults_filters_with_existing_state() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let mut store = BasicFilterStore::new();

        // Set initial filters (loaded from URL params / localStorage / etc)
        store.set_vaults_filters(GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: false,
            tokens: Some(vec![token1]),
        });

        // Then changing only part of the state
        store.update_vaults_filters(|builder| {
            builder.set_owners(vec![owner2]).set_hide_zero_balance(true)
        });

        assert_eq!(store.vaults.owners.len(), 1);
        assert_eq!(store.vaults.owners[0], owner2);
        assert!(store.vaults.hide_zero_balance);
        assert!(store.vaults.tokens.is_some());
        assert_eq!(store.vaults.tokens.as_ref().unwrap().len(), 1);
        assert_eq!(store.vaults.tokens.as_ref().unwrap()[0], token1);
    }

    #[test]
    fn test_immutability_original_filters_not_modified() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

        let mut store = BasicFilterStore::new();

        // Set initial filters
        let original_filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: false,
            tokens: None,
        };
        store.set_vaults_filters(original_filters.clone());
        let filters_before_update = store.vaults.clone();
        store.update_vaults_filters(|builder| {
            builder.set_owners(vec![owner2]).set_hide_zero_balance(true)
        });

        // Check that original filters are not modified
        assert_eq!(original_filters.owners.len(), 1);
        assert_eq!(original_filters.owners[0], owner1);
        assert!(!original_filters.hide_zero_balance);
        assert!(original_filters.tokens.is_none());

        // Check that filters stored before update are not modified
        assert_eq!(filters_before_update.owners.len(), 1);
        assert_eq!(filters_before_update.owners[0], owner1);
        assert!(!filters_before_update.hide_zero_balance);
        assert!(filters_before_update.tokens.is_none());

        // Check that store was updated correctly
        assert_eq!(store.vaults.owners.len(), 1);
        assert_eq!(store.vaults.owners[0], owner2);
        assert!(store.vaults.hide_zero_balance);
    }

    #[test]
    fn test_multiple_updates_dont_affect_each_other() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();
        let owner3 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let mut store = BasicFilterStore::new();

        store.update_vaults_filters(|builder| builder.set_owners(vec![owner1]));
        let state_after_first_update = store.vaults.clone();

        store.update_vaults_filters(|builder| {
            builder.set_owners(vec![owner2]).set_hide_zero_balance(true)
        });
        let state_after_second_update = store.vaults.clone();

        store.update_vaults_filters(|builder| {
            builder
                .set_owners(vec![owner3])
                .set_hide_zero_balance(false)
        });

        // Check that each update did not affect the previously "extracted" state
        assert_eq!(state_after_first_update.owners.len(), 1);
        assert_eq!(state_after_first_update.owners[0], owner1);
        assert!(!state_after_first_update.hide_zero_balance);

        assert_eq!(state_after_second_update.owners.len(), 1);
        assert_eq!(state_after_second_update.owners[0], owner2);
        assert!(state_after_second_update.hide_zero_balance);

        // Check that final store state is correct
        assert_eq!(store.vaults.owners.len(), 1);
        assert_eq!(store.vaults.owners[0], owner3);
        assert!(!store.vaults.hide_zero_balance);
    }

    #[test]
    fn test_url_params_integration() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let mut store = BasicFilterStore::new();

        store.update_vaults_filters(|builder| {
            builder
                .set_owners(vec![owner1])
                .set_hide_zero_balance(true)
                .set_tokens(Some(vec![token1]))
        });

        let url_params = store.vaults.to_url_params();
        assert!(url_params.contains(&format!("owner={}", owner1)));
        assert!(url_params.contains("hideZeroBalance=true"));
        assert!(url_params.contains(&format!("token[]={}", token1)));

        let restored_filters = GetVaultsFilters::from_url_params(url_params).unwrap();
        assert_eq!(store.vaults.owners, restored_filters.owners);
        assert_eq!(
            store.vaults.hide_zero_balance,
            restored_filters.hide_zero_balance
        );
        assert_eq!(store.vaults.tokens, restored_filters.tokens);
    }

    #[test]
    fn test_serialization_integration() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let mut store = BasicFilterStore::new();

        store.update_vaults_filters(|builder| {
            builder
                .set_owners(vec![owner1])
                .set_hide_zero_balance(true)
                .set_tokens(Some(vec![token1]))
        });

        // Test JSON serialization
        let json = serde_json::to_string(&store).unwrap();
        let deserialized: BasicFilterStore = serde_json::from_str(&json).unwrap();

        assert_eq!(store.vaults.owners, deserialized.vaults.owners);
        assert_eq!(
            store.vaults.hide_zero_balance,
            deserialized.vaults.hide_zero_balance
        );
        assert_eq!(store.vaults.tokens, deserialized.vaults.tokens);
    }
    #[test]
    fn test_filter_store_trait_implementation() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        // Test FilterStore trait
        let mut store = BasicFilterStore::new();

        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: None,
        };

        // Test set_vaults_filters
        FilterStore::set_vaults_filters(&mut store, filters.clone());
        assert_eq!(store.vaults.owners, filters.owners);
        assert_eq!(store.vaults.hide_zero_balance, filters.hide_zero_balance);
        assert_eq!(store.vaults.tokens, filters.tokens);

        // Test update_vaults_filters
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();
        FilterStore::update_vaults_filters(&mut store, |builder| builder.set_owners(vec![owner2]));
        assert_eq!(store.vaults.owners.len(), 1);
        assert_eq!(store.vaults.owners[0], owner2);
        assert!(store.vaults.hide_zero_balance); // should be preserved
    }

    #[test]
    fn test_clone_behavior() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        let mut store1 = BasicFilterStore::new();
        store1.update_vaults_filters(|builder| {
            builder.set_owners(vec![owner1]).set_hide_zero_balance(true)
        });

        // Clone store
        let mut store2 = store1.clone();

        // Modify clone
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();
        store2.update_vaults_filters(|builder| {
            builder
                .set_owners(vec![owner2])
                .set_hide_zero_balance(false)
        });

        // Check that original wasn't modified
        assert_eq!(store1.vaults.owners.len(), 1);
        assert_eq!(store1.vaults.owners[0], owner1);
        assert!(store1.vaults.hide_zero_balance);

        // Check that clone was modified
        assert_eq!(store2.vaults.owners.len(), 1);
        assert_eq!(store2.vaults.owners[0], owner2);
        assert!(!store2.vaults.hide_zero_balance);
    }
}
