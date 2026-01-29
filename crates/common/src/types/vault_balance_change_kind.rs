use serde::{Deserialize, Serialize};

const DEPOSIT_TYPES: &[&str] = &["DEPOSIT"];
const WITHDRAWAL_TYPES: &[&str] = &["WITHDRAW"];
const TAKE_ORDER_TYPES: &[&str] = &["TAKE_INPUT", "TAKE_OUTPUT"];
const CLEAR_TYPES: &[&str] = &[
    "CLEAR_ALICE_INPUT",
    "CLEAR_ALICE_OUTPUT",
    "CLEAR_BOB_INPUT",
    "CLEAR_BOB_OUTPUT",
];
const CLEAR_BOUNTY_TYPES: &[&str] = &["CLEAR_ALICE_BOUNTY", "CLEAR_BOB_BOUNTY"];
const UNKNOWN_TYPES: &[&str] = &[];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VaultBalanceChangeKind {
    Deposit,
    Withdrawal,
    TakeOrder,
    Clear,
    ClearBounty,
    Unknown,
}

impl VaultBalanceChangeKind {
    pub fn from_local_db_change_type(s: &str) -> Self {
        match s {
            "DEPOSIT" => Self::Deposit,
            "WITHDRAW" => Self::Withdrawal,
            "TAKE_INPUT" | "TAKE_OUTPUT" => Self::TakeOrder,
            "CLEAR_ALICE_INPUT" | "CLEAR_ALICE_OUTPUT" | "CLEAR_BOB_INPUT" | "CLEAR_BOB_OUTPUT" => {
                Self::Clear
            }
            "CLEAR_ALICE_BOUNTY" | "CLEAR_BOB_BOUNTY" => Self::ClearBounty,
            _ => Self::Unknown,
        }
    }

    pub fn from_local_db_trade_kind(s: &str) -> Self {
        match s {
            "take" => Self::TakeOrder,
            "clear" => Self::Clear,
            _ => Self::Unknown,
        }
    }

    pub fn from_subgraph_typename(s: &str) -> Self {
        match s {
            "Deposit" => Self::Deposit,
            "Withdrawal" => Self::Withdrawal,
            "TakeOrder" => Self::TakeOrder,
            "Clear" => Self::Clear,
            "ClearBounty" => Self::ClearBounty,
            _ => Self::Unknown,
        }
    }

    pub fn to_subgraph_typenames(&self) -> &'static [&'static str] {
        match self {
            Self::Deposit => &["Deposit"],
            Self::Withdrawal => &["Withdrawal"],
            Self::TakeOrder => &["TakeOrder", "TradeVaultBalanceChange"],
            Self::Clear => &["Clear", "TradeVaultBalanceChange"],
            Self::ClearBounty => &["ClearBounty"],
            Self::Unknown => &[],
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Deposit => "Deposit",
            Self::Withdrawal => "Withdrawal",
            Self::TakeOrder => "Take order",
            Self::Clear => "Clear",
            Self::ClearBounty => "Clear Bounty",
            Self::Unknown => "Unknown",
        }
    }

    pub fn to_local_db_change_types(&self) -> &'static [&'static str] {
        match self {
            Self::Deposit => DEPOSIT_TYPES,
            Self::Withdrawal => WITHDRAWAL_TYPES,
            Self::TakeOrder => TAKE_ORDER_TYPES,
            Self::Clear => CLEAR_TYPES,
            Self::ClearBounty => CLEAR_BOUNTY_TYPES,
            Self::Unknown => UNKNOWN_TYPES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_local_db_change_type() {
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("DEPOSIT"),
            VaultBalanceChangeKind::Deposit
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("WITHDRAW"),
            VaultBalanceChangeKind::Withdrawal
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("TAKE_INPUT"),
            VaultBalanceChangeKind::TakeOrder
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("TAKE_OUTPUT"),
            VaultBalanceChangeKind::TakeOrder
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("CLEAR_ALICE_INPUT"),
            VaultBalanceChangeKind::Clear
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("CLEAR_BOB_OUTPUT"),
            VaultBalanceChangeKind::Clear
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("CLEAR_ALICE_BOUNTY"),
            VaultBalanceChangeKind::ClearBounty
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("CLEAR_BOB_BOUNTY"),
            VaultBalanceChangeKind::ClearBounty
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_change_type("UNKNOWN_TYPE"),
            VaultBalanceChangeKind::Unknown
        );
    }

    #[test]
    fn test_from_local_db_trade_kind() {
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_trade_kind("take"),
            VaultBalanceChangeKind::TakeOrder
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_trade_kind("clear"),
            VaultBalanceChangeKind::Clear
        );
        assert_eq!(
            VaultBalanceChangeKind::from_local_db_trade_kind("unknown"),
            VaultBalanceChangeKind::Unknown
        );
    }

    #[test]
    fn test_from_subgraph_typename() {
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("Deposit"),
            VaultBalanceChangeKind::Deposit
        );
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("Withdrawal"),
            VaultBalanceChangeKind::Withdrawal
        );
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("TakeOrder"),
            VaultBalanceChangeKind::TakeOrder
        );
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("Clear"),
            VaultBalanceChangeKind::Clear
        );
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("ClearBounty"),
            VaultBalanceChangeKind::ClearBounty
        );
        assert_eq!(
            VaultBalanceChangeKind::from_subgraph_typename("Unknown"),
            VaultBalanceChangeKind::Unknown
        );
    }

    #[test]
    fn test_display_name() {
        assert_eq!(VaultBalanceChangeKind::Deposit.display_name(), "Deposit");
        assert_eq!(
            VaultBalanceChangeKind::Withdrawal.display_name(),
            "Withdrawal"
        );
        assert_eq!(
            VaultBalanceChangeKind::TakeOrder.display_name(),
            "Take order"
        );
        assert_eq!(VaultBalanceChangeKind::Clear.display_name(), "Clear");
        assert_eq!(
            VaultBalanceChangeKind::ClearBounty.display_name(),
            "Clear Bounty"
        );
        assert_eq!(VaultBalanceChangeKind::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_to_local_db_change_types() {
        assert_eq!(
            VaultBalanceChangeKind::Deposit.to_local_db_change_types(),
            &["DEPOSIT"]
        );
        assert_eq!(
            VaultBalanceChangeKind::Withdrawal.to_local_db_change_types(),
            &["WITHDRAW"]
        );
        assert_eq!(
            VaultBalanceChangeKind::TakeOrder.to_local_db_change_types(),
            &["TAKE_INPUT", "TAKE_OUTPUT"]
        );
        assert_eq!(
            VaultBalanceChangeKind::Clear.to_local_db_change_types(),
            &[
                "CLEAR_ALICE_INPUT",
                "CLEAR_ALICE_OUTPUT",
                "CLEAR_BOB_INPUT",
                "CLEAR_BOB_OUTPUT"
            ]
        );
        assert_eq!(
            VaultBalanceChangeKind::ClearBounty.to_local_db_change_types(),
            &["CLEAR_ALICE_BOUNTY", "CLEAR_BOB_BOUNTY"]
        );
        assert_eq!(
            VaultBalanceChangeKind::Unknown.to_local_db_change_types(),
            &[] as &[&str]
        );
    }
}
