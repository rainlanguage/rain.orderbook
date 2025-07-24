use super::traits::Filter;
use crate::raindex_client::*;
use alloy::primitives::Address;
use rain_orderbook_subgraph_client::types::common::{SgBytes, SgVaultsListFilterArgs};

//
// Vaults Filters
//

#[derive(Serialize, Deserialize, Debug, Clone, Tsify, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetVaultsFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    pub hide_zero_balance: bool,
    #[tsify(optional, type = "Address[]")]
    pub tokens: Option<Vec<Address>>,
    #[tsify(optional, type = "ChainIds")]
    pub chain_ids: Option<Vec<u64>>,
}
impl_wasm_traits!(GetVaultsFilters);

impl TryFrom<GetVaultsFilters> for SgVaultsListFilterArgs {
    type Error = RaindexError;
    fn try_from(filters: GetVaultsFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            owners: filters
                .owners
                .into_iter()
                .map(|owner| SgBytes(owner.to_string()))
                .collect(),
            hide_zero_balance: filters.hide_zero_balance,
            tokens: filters
                .tokens
                .map(|tokens| {
                    tokens
                        .into_iter()
                        .map(|token| token.to_string().to_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl Filter for GetVaultsFilters {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn test_get_vaults_filters_default() {
        let filters = GetVaultsFilters::default();
        assert!(filters.owners.is_empty());
        assert!(!filters.hide_zero_balance);
        assert!(filters.tokens.is_none());
        assert!(filters.chain_ids.is_none());
    }

    //
    // TryFrom conversion to SgVaultsListFilterArgs
    //
    #[test]
    fn test_try_from_sg_vaults_list_filter_args() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
            chain_ids: Some(vec![1, 137]),
        };

        let sg_filter_args: SgVaultsListFilterArgs = filters.try_into().unwrap();

        assert_eq!(sg_filter_args.owners.len(), 1);
        assert_eq!(sg_filter_args.owners[0].0, owner1.to_string());
        assert!(sg_filter_args.hide_zero_balance);
        assert_eq!(sg_filter_args.tokens.len(), 1);
        assert_eq!(sg_filter_args.tokens[0], token1.to_string().to_lowercase());
    }

    #[test]
    fn test_chain_ids_filter() {
        let filters = GetVaultsFilters {
            owners: vec![],
            hide_zero_balance: false,
            tokens: None,
            chain_ids: Some(vec![1, 137, 10]),
        };

        assert_eq!(filters.chain_ids.as_ref().unwrap().len(), 3);
        assert!(filters.chain_ids.as_ref().unwrap().contains(&1));
        assert!(filters.chain_ids.as_ref().unwrap().contains(&137));
        assert!(filters.chain_ids.as_ref().unwrap().contains(&10));
    }

    #[test]
    fn test_filters_without_chain_ids() {
        let filters = GetVaultsFilters {
            owners: vec![],
            hide_zero_balance: true,
            tokens: None,
            chain_ids: None,
        };

        assert!(filters.chain_ids.is_none());
        assert!(filters.hide_zero_balance);
    }
}
