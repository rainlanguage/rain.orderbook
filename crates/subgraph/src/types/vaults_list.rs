use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultsListQueryVariables {
    pub first: Option<i32>,
    pub skip: Option<i32>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "VaultsListQueryVariables")]
pub struct VaultsListQuery {
    #[arguments(orderBy: "id", orderDirection: "desc", skip: $skip, first: $first)]
    pub token_vaults: Vec<TokenVault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub owner: Account,
    pub vault_id: BigInt,
    pub token: Erc20,
    pub balance_display: BigDecimal,
    pub balance: BigInt,
    #[arguments(orderBy: "id", orderDirection: "desc")]
    pub orders: Option<Vec<Order>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Order {
    pub id: cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[typeshare]
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

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "TokenVault_orderBy")]
pub enum TokenVaultOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "owner")]
    Owner,
    #[cynic(rename = "owner__id")]
    OwnerId,
    #[cynic(rename = "vault")]
    Vault,
    #[cynic(rename = "vault__id")]
    VaultId,
    #[cynic(rename = "vault__vaultId")]
    VaultVaultId,
    #[cynic(rename = "vaultId")]
    VaultId2,
    #[cynic(rename = "token")]
    Token,
    #[cynic(rename = "token__id")]
    TokenId,
    #[cynic(rename = "token__name")]
    TokenName,
    #[cynic(rename = "token__symbol")]
    TokenSymbol,
    #[cynic(rename = "token__totalSupply")]
    TokenTotalSupply,
    #[cynic(rename = "token__totalSupplyDisplay")]
    TokenTotalSupplyDisplay,
    #[cynic(rename = "token__decimals")]
    TokenDecimals,
    #[cynic(rename = "balance")]
    Balance,
    #[cynic(rename = "balanceDisplay")]
    BalanceDisplay,
    #[cynic(rename = "orders")]
    Orders,
    #[cynic(rename = "orderClears")]
    OrderClears,
    #[cynic(rename = "takeOrders")]
    TakeOrders,
}

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
