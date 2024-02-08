use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultDetailQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "VaultDetailQueryVariables")]
pub struct VaultDetailQuery {
    #[arguments(id: $id)]
    pub token_vault: Option<TokenVault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(variables = "VaultDetailQueryVariables")]
pub struct TokenVault {
    pub id: cynic::Id,
    pub owner: Account,
    pub balance: BigInt,
    pub balance_display: BigDecimal,
    pub token: Erc20,
    pub vault_id: BigInt,
    pub vault: Vault,
    #[arguments(orderBy: "id", orderDirection: "desc")]
    pub orders: Option<Vec<Order>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(variables = "VaultDetailQueryVariables")]
pub struct Vault {
    pub id: cynic::Id,
    #[arguments(where: { tokenVault_: { id: $id } })]
    pub deposits: Option<Vec<VaultDeposit>>,
    #[arguments(where: { tokenVault_: { id: $id } })]
    pub withdraws: Option<Vec<VaultWithdraw>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct VaultWithdraw {
    pub id: cynic::Id,
    pub sender: Account,
    pub transaction: Transaction,
    pub timestamp: BigInt,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub requested_amount: BigInt,
    pub requested_amount_display: BigDecimal,
    pub token_vault: TokenVault2,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "TokenVault")]
pub struct TokenVault2 {
    pub balance_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub transaction: Transaction,
    pub timestamp: BigInt,
    pub sender: Account,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Transaction {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Order {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Order_orderBy")]
pub enum OrderOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "orderHash")]
    OrderHash,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "owner__id")]
    OwnerId,
    #[cynic(rename = "interpreter")]
    Interpreter,
    #[cynic(rename = "interpreterStore")]
    InterpreterStore,
    #[cynic(rename = "expressionDeployer")]
    ExpressionDeployer,
    #[cynic(rename = "expression")]
    Expression,
    #[cynic(rename = "orderActive")]
    OrderActive,
    #[cynic(rename = "handleIO")]
    HandleIo,
    #[cynic(rename = "meta")]
    Meta,
    #[cynic(rename = "meta__id")]
    MetaId,
    #[cynic(rename = "meta__metaBytes")]
    MetaMetaBytes,
    #[cynic(rename = "validInputs")]
    ValidInputs,
    #[cynic(rename = "validOutputs")]
    ValidOutputs,
    #[cynic(rename = "orderJSONString")]
    OrderJsonstring,
    #[cynic(rename = "expressionJSONString")]
    ExpressionJsonstring,
    #[cynic(rename = "transaction")]
    Transaction,
    #[cynic(rename = "transaction__id")]
    TransactionId,
    #[cynic(rename = "transaction__timestamp")]
    TransactionTimestamp,
    #[cynic(rename = "transaction__blockNumber")]
    TransactionBlockNumber,
    #[cynic(rename = "emitter")]
    Emitter,
    #[cynic(rename = "emitter__id")]
    EmitterId,
    #[cynic(rename = "timestamp")]
    Timestamp,
    #[cynic(rename = "takeOrders")]
    TakeOrders,
    #[cynic(rename = "ordersClears")]
    OrdersClears,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
