use super::super::traits::Filter;
use crate::raindex_client::*;
use alloy::primitives::{Address, Bytes};
use rain_orderbook_subgraph_client::types::common::{SgBytes, SgOrdersListFilterArgs};

//
// Orders Filters
//

#[derive(Serialize, Deserialize, Debug, Clone, Tsify, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    #[tsify(optional)]
    pub active: Option<bool>,
    #[tsify(optional, type = "Hex")]
    pub order_hash: Option<Bytes>,
    #[tsify(optional, type = "Address[]")]
    pub tokens: Option<Vec<Address>>,
    #[tsify(optional, type = "ChainIds")]
    pub chain_ids: Option<Vec<u32>>,
}
impl_wasm_traits!(GetOrdersFilters);

impl TryFrom<GetOrdersFilters> for SgOrdersListFilterArgs {
    type Error = RaindexError;
    fn try_from(filters: GetOrdersFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            owners: filters
                .owners
                .into_iter()
                .map(|owner| SgBytes(owner.to_string()))
                .collect(),
            active: filters.active,
            order_hash: filters
                .order_hash
                .map(|order_hash| SgBytes(order_hash.to_string())),
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

impl Filter for GetOrdersFilters {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_orders_filters_try_from() {
        let filters = GetOrdersFilters {
            owners: vec!["0x1234567890abcdef1234567890abcdef12345678"
                .parse()
                .unwrap()],
            active: Some(true),
            order_hash: Some(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                    .parse()
                    .unwrap(),
            ),
            tokens: Some(vec!["0xfedcba0987654321fedcba0987654321fedcba09"
                .parse()
                .unwrap()]),
            chain_ids: Some(vec![1, 137]),
        };

        let sg_filters: SgOrdersListFilterArgs = filters.try_into().unwrap();

        assert_eq!(sg_filters.owners.len(), 1);
        assert_eq!(sg_filters.active, Some(true));
        assert!(sg_filters.order_hash.is_some());
        assert_eq!(sg_filters.tokens.len(), 1);
    }

    #[test]
    fn test_get_orders_filters_default() {
        let filters = GetOrdersFilters::default();
        assert!(filters.owners.is_empty());
        assert!(filters.active.is_none());
        assert!(filters.order_hash.is_none());
        assert!(filters.tokens.is_none());
        assert!(filters.chain_ids.is_none());
    }
}
