use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug, Clone)]
#[typeshare]
pub struct VaultDetailQueryVariables {
    pub id: Bytes,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Withdrawal {
    pub id: Bytes,
    pub __typename: String,
    pub amount: BigInt,
    pub old_vault_balance: BigInt,
    pub new_vault_balance: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct TradeVaultBalanceChange {
    pub id: Bytes,
    pub __typename: String,
    pub amount: BigInt,
    pub old_vault_balance: BigInt,
    pub new_vault_balance: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultDetailQueryVariables")]
#[typeshare]
pub struct VaultDetailQuery {
    #[arguments(id: $id)]
    pub vault: Option<Vault>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Vault {
    pub vault_id: BigInt,
    pub token: Bytes,
    pub owner: Bytes,
    pub orders_as_ouput: Vec<Order>,
    pub orders_as_input: Vec<Order>,
    pub balance_changes: Vec<VaultBalanceChange>,
    pub balance: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Order {
    pub order_hash: Bytes,
    pub active: bool,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[typeshare]
pub struct Deposit {
    pub id: Bytes,
    pub __typename: String,
    pub amount: BigInt,
    pub old_vault_balance: BigInt,
    pub new_vault_balance: BigInt,
}

#[derive(cynic::InlineFragments, Debug, Clone, Serialize)]
#[typeshare]
pub enum VaultBalanceChange {
    Withdrawal(Withdrawal),
    TradeVaultBalanceChange(TradeVaultBalanceChange),
    Deposit(Deposit),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[typeshare]
pub struct Bytes(pub String);
