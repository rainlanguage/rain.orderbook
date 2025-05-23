use crate::csv::TryIntoCsv;
use alloy::primitives::{utils::format_units, U256};
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: SgBytes,
    pub vault_id: SgBigInt,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<SgBigInt>,
    pub token_address: String,
    pub balance_display: String,
    pub balance: SgBigInt,
}

impl TryFrom<SgVault> for TokenVaultFlattened {
    type Error = FlattenError;

    fn try_from(val: SgVault) -> Result<Self, Self::Error> {
        let balance_parsed = val.balance.0.parse::<U256>()?;
        let decimals = val
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
            .0
            .parse::<u8>()?;

        Ok(Self {
            id: val.id.0,
            owner: val.owner,
            vault_id: val.vault_id,
            token_name: val.token.name,
            token_symbol: val.token.symbol,
            token_decimals: val.token.decimals,
            token_address: val.token.address.0,
            balance_display: format_units(balance_parsed, decimals)?,
            balance: val.balance,
        })
    }
}

impl TryIntoCsv<TokenVaultFlattened> for Vec<TokenVaultFlattened> {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;
    use rain_orderbook_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgVault, SgVaultBalanceChangeType,
    };

    #[allow(clippy::too_many_arguments)]
    fn create_sg_vault(
        id: &str,
        owner: &str,
        vault_id_str: &str,
        balance_str: &str,
        token_id: &str,
        token_address: &str,
        token_name: Option<&str>,
        token_symbol: Option<&str>,
        token_decimals_str: Option<&str>,
    ) -> SgVault {
        SgVault {
            id: SgBytes(id.into()),
            owner: SgBytes(owner.into()),
            vault_id: SgBigInt(vault_id_str.into()),
            balance: SgBigInt(balance_str.into()),
            token: SgErc20 {
                id: SgBytes(token_id.into()),
                address: SgBytes(token_address.into()),
                name: token_name.map(String::from),
                symbol: token_symbol.map(String::from),
                decimals: token_decimals_str.map(|s| SgBigInt(s.into())),
            },
            orderbook: SgOrderbook {
                id: SgBytes("default_orderbook_id".into()),
            },
            orders_as_output: Vec::<SgOrderAsIO>::new(),
            orders_as_input: Vec::<SgOrderAsIO>::new(),
            balance_changes: Vec::<SgVaultBalanceChangeType>::new(),
        }
    }

    #[test]
    fn test_normal_case_all_fields_present() {
        let sg_vault = create_sg_vault(
            "vault_test_001",
            "0xOwnerAddress1",
            "1001",
            "123450000000000000000",
            "token_test_TKA",
            "0xTokenAddressA",
            Some("Test Token A"),
            Some("TKA"),
            Some("18"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.id, "vault_test_001");
        assert_eq!(flattened.owner, SgBytes("0xOwnerAddress1".into()));
        assert_eq!(flattened.vault_id, SgBigInt("1001".into()));
        assert_eq!(flattened.token_name, Some("Test Token A".into()));
        assert_eq!(flattened.token_symbol, Some("TKA".into()));
        assert_eq!(flattened.token_decimals, Some(SgBigInt("18".into())));
        assert_eq!(flattened.token_address, "0xTokenAddressA");
        assert_eq!(flattened.balance_display, "123.450000000000000000");
        assert_eq!(flattened.balance, SgBigInt("123450000000000000000".into()));
    }

    #[test]
    fn test_normal_case_optional_token_fields_none() {
        let sg_vault = create_sg_vault(
            "vault_test_002",
            "0xOwnerAddress2",
            "1002",
            "7890",
            "token_test_TKB",
            "0xTokenAddressB",
            None,
            None,
            None,
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.token_name, None);
        assert_eq!(flattened.token_symbol, None);
        assert_eq!(flattened.token_decimals, None);
        assert_eq!(flattened.balance_display, "7890.0");
        assert_eq!(flattened.balance, SgBigInt("7890".into()));
    }

    #[test]
    fn test_edge_case_zero_balance() {
        let sg_vault = create_sg_vault(
            "vault_test_003",
            "0xOwnerAddress3",
            "1003",
            "0",
            "token_test_TKC",
            "0xTokenAddressC",
            Some("Test Token C"),
            Some("TKC"),
            Some("18"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.balance_display, "0.000000000000000000");
        assert_eq!(flattened.balance, SgBigInt("0".into()));
    }

    #[test]
    fn test_edge_case_zero_balance_zero_decimals_explicit() {
        let sg_vault = create_sg_vault(
            "vault_test_003b",
            "0xOwnerAddress3b",
            "10032",
            "0",
            "token_test_TKC2",
            "0xTokenAddressC2",
            Some("Test Token C2"),
            Some("TKC2"),
            Some("0"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.balance_display, "0.0");
        assert_eq!(flattened.balance, SgBigInt("0".into()));
    }

    #[test]
    fn test_edge_case_zero_decimals_specified() {
        let sg_vault = create_sg_vault(
            "vault_test_004",
            "0xOwnerAddress4",
            "1004",
            "98765",
            "token_test_TKD",
            "0xTokenAddressD",
            Some("Test Token D"),
            Some("TKD"),
            Some("0"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.balance_display, "98765.0");
        assert_eq!(flattened.balance, SgBigInt("98765".into()));
    }

    #[test]
    fn test_boundary_case_large_balance_u256_max() {
        let u256_max_str = U256::MAX.to_string();
        let sg_vault = create_sg_vault(
            "vault_test_005",
            "0xOwnerAddress5",
            "1005",
            &u256_max_str,
            "token_test_TKE",
            "0xTokenAddressE",
            Some("Test Token E"),
            Some("TKE"),
            Some("0"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(
            flattened.balance_display,
            "115792089237316195423570985008687907853269984665640564039457584007913129639935.0"
        );
        assert_eq!(flattened.balance, SgBigInt(u256_max_str));
    }

    #[test]
    fn test_boundary_case_large_balance_u256_max_with_decimals() {
        let balance_val = "123456000000000000000000";
        let sg_vault = create_sg_vault(
            "vault_test_005b",
            "0xOwnerAddress5b",
            "10052",
            balance_val,
            "token_test_TKE2",
            "0xTokenAddressE2",
            Some("Test Token E2"),
            Some("TKE2"),
            Some("18"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.balance_display, "123456.000000000000000000");
        assert_eq!(flattened.balance, SgBigInt(balance_val.to_string()));
    }

    #[test]
    fn test_error_case_invalid_balance_non_numeric() {
        let sg_vault = create_sg_vault(
            "vault_test_006",
            "0xOwnerAddress6",
            "1006",
            "not-a-number",
            "token_test_TKF",
            "0xTokenAddressF",
            None,
            None,
            Some("18"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_case_invalid_balance_negative() {
        let sg_vault = create_sg_vault(
            "vault_test_007",
            "0xOwnerAddress7",
            "1007",
            "-1000",
            "token_test_TKG",
            "0xTokenAddressG",
            None,
            None,
            Some("18"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_case_invalid_decimals_non_numeric() {
        let sg_vault = create_sg_vault(
            "vault_test_008",
            "0xOwnerAddress8",
            "1008",
            "1000",
            "token_test_TKH",
            "0xTokenAddressH",
            None,
            None,
            Some("eighteen"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_case_decimals_out_of_u8_range_positive() {
        let sg_vault = create_sg_vault(
            "vault_test_009",
            "0xOwnerAddress9",
            "1009",
            "1000",
            "token_test_TKI",
            "0xTokenAddressI",
            None,
            None,
            Some("256"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_case_decimals_out_of_u8_range_negative() {
        let sg_vault = create_sg_vault(
            "vault_test_010",
            "0xOwnerAddress10",
            "1010",
            "1000",
            "token_test_TKJ",
            "0xTokenAddressJ",
            None,
            None,
            Some("-1"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_case_format_units_decimals_too_large() {
        let sg_vault = create_sg_vault(
            "vault_test_011",
            "0xOwnerAddress11",
            "1011",
            "1000000000000000000",
            "token_test_TKK",
            "0xTokenAddressK",
            None,
            None,
            Some("78"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_allowed_decimals_for_format_units() {
        let balance_str = "1".to_string() + &"0".repeat(77);
        let sg_vault = create_sg_vault(
            "vault_test_012",
            "0xOwnerAddress12",
            "1012",
            &balance_str,
            "token_test_TKL",
            "0xTokenAddressL",
            None,
            None,
            Some("77"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(
            flattened.balance_display,
            "1.00000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
