use super::traits::FilterBuilder;
use super::vaults_filter::GetVaultsFilters;
use crate::raindex_client::*;
use alloy::primitives::Address;

//
// Vaults Filter Builder
//

#[derive(Serialize, Deserialize, Debug, Clone, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct VaultsFilterBuilder {
    pub owners: Vec<Address>,
    pub hide_zero_balance: bool,
    pub tokens: Option<Vec<Address>>,
    pub chain_ids: Option<Vec<u64>>,
}
impl_wasm_traits!(VaultsFilterBuilder);

impl VaultsFilterBuilder {
    pub fn new() -> Self {
        Self {
            owners: Vec::new(),
            hide_zero_balance: false,
            tokens: None,
            chain_ids: None,
        }
    }

    pub fn from(filters: GetVaultsFilters) -> Self {
        Self {
            owners: filters.owners.clone(),
            hide_zero_balance: filters.hide_zero_balance,
            tokens: filters.tokens.clone(),
            chain_ids: filters.chain_ids.clone(),
        }
    }

    pub fn set_owners(mut self, owners: Vec<Address>) -> Self {
        self.owners = owners;
        self
    }

    pub fn set_hide_zero_balance(mut self, hide_zero_balance: bool) -> Self {
        self.hide_zero_balance = hide_zero_balance;
        self
    }

    pub fn set_tokens(mut self, tokens: Option<Vec<Address>>) -> Self {
        self.tokens = tokens;
        self
    }

    pub fn set_chain_ids(mut self, chain_ids: Option<Vec<u64>>) -> Self {
        self.chain_ids = chain_ids;
        self
    }
}

impl FilterBuilder for VaultsFilterBuilder {
    type Output = GetVaultsFilters;

    fn build(self) -> Self::Output {
        GetVaultsFilters {
            owners: self.owners,
            hide_zero_balance: self.hide_zero_balance,
            tokens: self.tokens,
            chain_ids: self.chain_ids,
        }
    }
}

impl From<VaultsFilterBuilder> for GetVaultsFilters {
    fn from(builder: VaultsFilterBuilder) -> Self {
        builder.build()
    }
}
impl From<GetVaultsFilters> for VaultsFilterBuilder {
    fn from(filters: GetVaultsFilters) -> Self {
        VaultsFilterBuilder::from(filters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::filters::traits::FilterBuilder;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn test_vaults_filter_builder_from_filters() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
            chain_ids: Some(vec![1, 137]),
        };

        let builder = VaultsFilterBuilder::from(filters.clone());
        assert_eq!(builder.owners, filters.owners);
        assert_eq!(builder.hide_zero_balance, filters.hide_zero_balance);
        assert_eq!(builder.tokens, filters.tokens);
        assert_eq!(builder.chain_ids, filters.chain_ids);
    }

    #[test]
    fn test_vaults_filter_builder_set_owners() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

        let builder = VaultsFilterBuilder::new().set_owners(vec![owner1, owner2]);

        assert_eq!(builder.owners.len(), 2);
        assert!(builder.owners.contains(&owner1));
        assert!(builder.owners.contains(&owner2));
    }

    #[test]
    fn test_vaults_filter_builder_chaining() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let builder = VaultsFilterBuilder::new()
            .set_owners(vec![owner1])
            .set_hide_zero_balance(true)
            .set_tokens(Some(vec![token1]));

        assert_eq!(builder.owners.len(), 1);
        assert_eq!(builder.owners[0], owner1);
        assert!(builder.hide_zero_balance);
        assert!(builder.tokens.is_some());
        assert_eq!(builder.tokens.as_ref().unwrap().len(), 1);
        assert_eq!(builder.tokens.as_ref().unwrap()[0], token1);
    }

    #[test]
    fn test_vaults_filter_builder_build() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let filters = VaultsFilterBuilder::new()
            .set_owners(vec![owner1])
            .set_hide_zero_balance(true)
            .set_tokens(Some(vec![token1]))
            .build();

        assert_eq!(filters.owners.len(), 1);
        assert_eq!(filters.owners[0], owner1);
        assert!(filters.hide_zero_balance);
        assert!(filters.tokens.is_some());
        assert_eq!(filters.tokens.as_ref().unwrap().len(), 1);
        assert_eq!(filters.tokens.as_ref().unwrap()[0], token1);
    }

    #[test]
    fn test_from_trait_vaults_filter_builder_to_filters() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        let builder = VaultsFilterBuilder::new()
            .set_owners(vec![owner1])
            .set_hide_zero_balance(true);

        let filters: GetVaultsFilters = builder.into();
        assert_eq!(filters.owners.len(), 1);
        assert_eq!(filters.owners[0], owner1);
        assert!(filters.hide_zero_balance);
    }

    #[test]
    fn test_from_trait_filters_to_vaults_filter_builder() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: None,
            chain_ids: None,
        };

        let builder: VaultsFilterBuilder = filters.into();
        assert_eq!(builder.owners.len(), 1);
        assert_eq!(builder.owners[0], owner1);
        assert!(builder.hide_zero_balance);
        assert!(builder.tokens.is_none());
    }

    #[test]
    fn test_builder_immutability() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

        let original_builder = VaultsFilterBuilder::new().set_owners(vec![owner1]);
        let new_builder = original_builder.clone().set_owners(vec![owner2]);

        // Check that original builder did not change
        assert_eq!(original_builder.owners.len(), 1);
        assert_eq!(original_builder.owners[0], owner1);

        // Check that new builder has new values
        assert_eq!(new_builder.owners.len(), 1);
        assert_eq!(new_builder.owners[0], owner2);
    }

    #[test]
    fn test_roundtrip_filters_to_builder_and_back() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let original_filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
            chain_ids: Some(vec![1, 10, 137]),
        };

        // Filters -> Builder -> Filters
        let builder = VaultsFilterBuilder::from(original_filters.clone());
        let restored_filters = builder.build();

        assert_eq!(original_filters.owners, restored_filters.owners);
        assert_eq!(
            original_filters.hide_zero_balance,
            restored_filters.hide_zero_balance
        );
        assert_eq!(original_filters.tokens, restored_filters.tokens);
        assert_eq!(original_filters.chain_ids, restored_filters.chain_ids);
    }
}
