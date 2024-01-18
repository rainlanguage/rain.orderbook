use crate::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct OrdersQuery {
    #[arguments(orderBy: "timestamp", orderDirection: "desc")]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Order {
    pub id: cynic::Id,
    pub timestamp: BigInt,
    #[cynic(rename = "handleIO")]
    pub handle_io: bool,
    #[cynic(rename = "orderJSONString")]
    pub order_jsonstring: String,
    pub owner: Account,
    pub order_active: bool,
    pub expression: Bytes,
    pub interpreter: Bytes,
    pub interpreter_store: Bytes,
    pub transaction: Transaction,
    pub valid_inputs: Option<Vec<Io>>,
    pub valid_outputs: Option<Vec<Io>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Transaction {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "IO")]
pub struct Io {
    pub token: Erc20,
    pub token_vault: TokenVault,
    pub vault: Vault,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Vault {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TokenVault {
    pub balance: BigInt,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ERC20")]
pub struct Erc20 {
    pub id: cynic::Id,
    pub symbol: String,
    pub decimals: i32,
}

#[derive(cynic::QueryFragment, Debug)]
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
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
