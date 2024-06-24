use crate::schema;
use typeshare::typeshare;
#[derive(cynic::QueryVariables, Debug)]
pub struct OrderTakeDetailQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrderTakeDetailQueryVariables")]
pub struct OrderTakeDetailQuery {
    #[arguments(id: $id)]
    pub trade: Option<Trade>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Trade {
    pub trade_event: TradeEvent,
    pub output_vault_balance_change: TradeVaultBalanceChange,
    pub order: Order,
    pub input_vault_balance_change: TradeVaultBalanceChange2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TradeVaultBalanceChange")]
pub struct TradeVaultBalanceChange2 {
    pub vault: Vault,
    pub amount: BigInt,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TradeVaultBalanceChange {
    pub amount: BigInt,
    pub vault: Vault,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub token: Bytes,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TradeEvent {
    pub transaction: Transaction,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Transaction {
    pub id: Bytes,
    pub from: Bytes,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Order {
    pub order_hash: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
