use super::traits::Filter;
use crate::raindex_client::{filters::errors::FilterError, *};
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

impl Filter for GetVaultsFilters {
    fn to_url_params(&self) -> String {
        let mut params = Vec::new();

        for owner in &self.owners {
            params.push(format!("owner={}", owner));
        }

        if self.hide_zero_balance {
            params.push("hideZeroBalance=true".to_string());
        }

        if let Some(tokens) = &self.tokens {
            for token in tokens {
                params.push(format!("token[]={}", token));
            }
        }

        params.join("&")
    }

    fn from_url_params(params: String) -> Result<Self, FilterError> {
        let mut owners = Vec::new();
        let mut hide_zero_balance = false;
        let mut tokens = None;

        if params.is_empty() {
            return Ok(GetVaultsFilters::default());
        }

        for param in params.split('&') {
            if param.is_empty() {
                continue;
            }

            let parts: Vec<&str> = param.splitn(2, '=').collect();
            if parts.len() == 1 {
                // Handle shorthand for hideZeroBalance
                if parts[0] == "hideZeroBalance" {
                    hide_zero_balance = true;
                    continue;
                }
            }
            if parts.len() != 2 {
                // Skip other cases
                continue;
            }

            let key = parts[0];
            let value = parts[1];

            match key {
                "owner" => {
                    let address = value.parse::<Address>().map_err(|err| {
                        FilterError::InvalidUrlParams(format!(
                            "Invalid owner address '{}': {}",
                            value, err
                        ))
                    })?;
                    owners.push(address);
                }
                "hideZeroBalance" => {
                    hide_zero_balance = value == "true";
                }
                "token[]" => {
                    if tokens.is_none() {
                        tokens = Some(Vec::new());
                    }
                    if let Some(ref mut t) = tokens {
                        let token = value.parse::<Address>().map_err(|err| {
                            FilterError::InvalidUrlParams(format!(
                                "Invalid token address '{}': {}",
                                value, err
                            ))
                        })?;
                        t.push(token);
                    }
                }
                _ => {
                    // Ignore unknown parameters
                }
            }
        }

        Ok(GetVaultsFilters {
            owners,
            hide_zero_balance,
            tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::filters::traits::Filter;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn test_get_vaults_filters_default() {
        let filters = GetVaultsFilters::default();
        assert!(filters.owners.is_empty());
        assert!(!filters.hide_zero_balance);
        assert!(filters.tokens.is_none());
    }

    //
    // URL Params serialization
    //
    #[test]
    fn test_to_url_params_empty() {
        let filters = GetVaultsFilters::default();
        let params = filters.to_url_params();
        assert_eq!(params, "");
    }

    #[test]
    fn test_to_url_params_with_owners_only() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![owner1, owner2],
            hide_zero_balance: false,
            tokens: None,
        };

        let params = filters.to_url_params();
        assert!(params.contains(&format!("owner={}", owner1)));
        assert!(params.contains(&format!("owner={}", owner2)));
        assert!(params.contains("&"));
    }

    #[test]
    fn test_to_url_params_with_hide_zero_balance() {
        let filters = GetVaultsFilters {
            owners: vec![],
            hide_zero_balance: true,
            tokens: None,
        };

        let params = filters.to_url_params();
        assert_eq!(params, "hideZeroBalance=true");
    }

    #[test]
    fn test_to_url_params_with_tokens() {
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();
        let token2 = Address::from_str("0x2222222222222222222222222222222222222222").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![],
            hide_zero_balance: false,
            tokens: Some(vec![token1, token2]),
        };

        let params = filters.to_url_params();
        assert!(params.contains(&format!("token[]={}", token1)));
        assert!(params.contains(&format!("token[]={}", token2)));
        assert!(params.contains("&"));
    }

    #[test]
    fn test_to_url_params_all() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
        };

        let params = filters.to_url_params();
        assert!(params.contains(&format!("owner={}", owner1)));
        assert!(params.contains("hideZeroBalance=true"));
        assert!(params.contains(&format!("token[]={}", token1)));
        // Should have 2 ampersands connecting 3 parameters
        assert_eq!(params.matches('&').count(), 2);
    }

    //
    // URL Params deserialization
    //
    #[test]
    fn test_from_url_params_empty() {
        let params = "".to_string();
        let filters = GetVaultsFilters::from_url_params(params).unwrap();

        assert!(filters.owners.is_empty());
        assert!(!filters.hide_zero_balance);
        assert!(filters.tokens.is_none());
    }

    #[test]
    fn test_from_url_params_with_owners() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let owner2 = Address::from_str("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd").unwrap();

        let params = format!("owner={}&owner={}", owner1, owner2);

        let filters = GetVaultsFilters::from_url_params(params).unwrap();
        assert_eq!(filters.owners.len(), 2);
        assert!(filters.owners.contains(&owner1));
        assert!(filters.owners.contains(&owner2));
    }

    #[test]
    fn test_from_url_params_with_hide_zero_balance() {
        let params = "hideZeroBalance=true".to_string();

        let filters = GetVaultsFilters::from_url_params(params).unwrap();
        assert!(filters.hide_zero_balance);
    }
    #[test]
    fn test_from_url_params_with_hide_zero_balance_shorthand() {
        let params = "hideZeroBalance".to_string();

        let filters = GetVaultsFilters::from_url_params(params).unwrap();
        assert!(filters.hide_zero_balance);
    }

    #[test]
    fn test_from_url_params_with_tokens() {
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();
        let token2 = Address::from_str("0x2222222222222222222222222222222222222222").unwrap();

        let params = format!("token[]={}&token[]={}", token1, token2);

        let filters = GetVaultsFilters::from_url_params(params).unwrap();
        assert!(filters.tokens.is_some());
        let tokens = filters.tokens.unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&token1));
        assert!(tokens.contains(&token2));
    }

    #[test]
    fn test_from_url_params_with_invalid_owner() {
        let params = "owner=invalid_address".to_string();

        let result = GetVaultsFilters::from_url_params(params);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid owner address"));
    }

    #[test]
    fn test_from_url_params_all() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let params = format!(
            "owner={}&hideZeroBalance=true&token[]={}&unknown_param=value",
            owner1, token1
        );

        let filters = GetVaultsFilters::from_url_params(params).unwrap();
        assert_eq!(filters.owners.len(), 1);
        assert_eq!(filters.owners[0], owner1);
        assert!(filters.hide_zero_balance);
        assert!(filters.tokens.is_some());
        let tokens = filters.tokens.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], token1);
    }

    #[test]
    fn test_from_url_params_malformed() {
        // Test various malformed parameter strings
        let test_cases = vec![
            "owner",                      // Missing value
            "=value",                     // Missing key
            "owner=",                     // Empty value (should fail for address parsing)
            "owner&hideZeroBalance=true", // Missing = for first param
        ];

        for malformed_params in test_cases {
            let result = GetVaultsFilters::from_url_params(malformed_params.to_string());
            // Some cases should succeed (like missing values), others should fail
            // The important thing is that they don't panic
            if malformed_params == "owner=" {
                assert!(
                    result.is_err(),
                    "Expected error for empty address: {}",
                    malformed_params
                );
            }
        }
    }

    #[test]
    fn test_roundtrip_url_params() {
        let owner1 = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let token1 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        let original_filters = GetVaultsFilters {
            owners: vec![owner1],
            hide_zero_balance: true,
            tokens: Some(vec![token1]),
        };

        // Convert to URL params and back
        let params = original_filters.to_url_params();
        let restored_filters = GetVaultsFilters::from_url_params(params).unwrap();

        assert_eq!(original_filters.owners, restored_filters.owners);
        assert_eq!(
            original_filters.hide_zero_balance,
            restored_filters.hide_zero_balance
        );
        assert_eq!(original_filters.tokens, restored_filters.tokens);
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
        };

        let sg_filter_args: SgVaultsListFilterArgs = filters.try_into().unwrap();

        assert_eq!(sg_filter_args.owners.len(), 1);
        assert_eq!(sg_filter_args.owners[0].0, owner1.to_string());
        assert!(sg_filter_args.hide_zero_balance);
        assert_eq!(sg_filter_args.tokens.len(), 1);
        assert_eq!(sg_filter_args.tokens[0], token1.to_string().to_lowercase());
    }
}
