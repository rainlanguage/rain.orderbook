use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct OrderTakesListQueryVariables<'a> {
    pub first: Option<i32>,
    pub id: &'a cynic::Id,
    pub skip: Option<i32>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrderTakesListQueryVariables")]
pub struct OrderTakesListQuery {
    #[arguments(orderBy: "timestamp", orderDirection: "desc", skip: $skip, first: $first, where: { order_: { id: $id } })]
    pub take_order_entities: Vec<TakeOrderEntity>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct TakeOrderEntity {
    pub id: cynic::Id,
    pub transaction: Transaction,
    pub sender: Account,
    pub timestamp: BigInt,
    pub order: Order,
    #[cynic(rename = "IORatio")]
    pub ioratio: BigDecimal,
    pub input: BigInt,
    pub input_display: BigDecimal,
    pub input_token: Erc20,
    #[cynic(rename = "inputIOIndex")]
    pub input_ioindex: BigInt,
    pub output: BigInt,
    pub output_display: BigDecimal,
    pub output_token: Erc20,
    #[cynic(rename = "outputIOIndex")]
    pub output_ioindex: BigInt,
    pub context: Option<ContextEntity>,
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
pub struct ContextEntity {
    pub calling_context: Option<Vec<BigInt>>,
    pub calculations_context: Option<Vec<BigInt>>,
    pub vault_inputs_context: Option<Vec<BigInt>>,
    pub vault_outputs_context: Option<Vec<BigInt>>,
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
#[cynic(graphql_type = "TakeOrderEntity_orderBy")]
pub enum TakeOrderEntityOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "sender")]
    Sender,
    #[cynic(rename = "sender__id")]
    SenderId,
    #[cynic(rename = "order")]
    Order,
    #[cynic(rename = "order__id")]
    OrderId,
    #[cynic(rename = "order__orderHash")]
    OrderOrderHash,
    #[cynic(rename = "order__interpreter")]
    OrderInterpreter,
    #[cynic(rename = "order__interpreterStore")]
    OrderInterpreterStore,
    #[cynic(rename = "order__expressionDeployer")]
    OrderExpressionDeployer,
    #[cynic(rename = "order__expression")]
    OrderExpression,
    #[cynic(rename = "order__orderActive")]
    OrderOrderActive,
    #[cynic(rename = "order__handleIO")]
    OrderHandleIo,
    #[cynic(rename = "order__orderJSONString")]
    OrderOrderJsonstring,
    #[cynic(rename = "order__expressionJSONString")]
    OrderExpressionJsonstring,
    #[cynic(rename = "order__timestamp")]
    OrderTimestamp,
    #[cynic(rename = "input")]
    Input,
    #[cynic(rename = "inputDisplay")]
    InputDisplay,
    #[cynic(rename = "output")]
    Output,
    #[cynic(rename = "outputDisplay")]
    OutputDisplay,
    #[cynic(rename = "IORatio")]
    Ioratio,
    #[cynic(rename = "inputIOIndex")]
    InputIoindex,
    #[cynic(rename = "outputIOIndex")]
    OutputIoindex,
    #[cynic(rename = "inputToken")]
    InputToken,
    #[cynic(rename = "inputToken__id")]
    InputTokenId,
    #[cynic(rename = "inputToken__name")]
    InputTokenName,
    #[cynic(rename = "inputToken__symbol")]
    InputTokenSymbol,
    #[cynic(rename = "inputToken__totalSupply")]
    InputTokenTotalSupply,
    #[cynic(rename = "inputToken__totalSupplyDisplay")]
    InputTokenTotalSupplyDisplay,
    #[cynic(rename = "inputToken__decimals")]
    InputTokenDecimals,
    #[cynic(rename = "outputToken")]
    OutputToken,
    #[cynic(rename = "outputToken__id")]
    OutputTokenId,
    #[cynic(rename = "outputToken__name")]
    OutputTokenName,
    #[cynic(rename = "outputToken__symbol")]
    OutputTokenSymbol,
    #[cynic(rename = "outputToken__totalSupply")]
    OutputTokenTotalSupply,
    #[cynic(rename = "outputToken__totalSupplyDisplay")]
    OutputTokenTotalSupplyDisplay,
    #[cynic(rename = "outputToken__decimals")]
    OutputTokenDecimals,
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
    #[cynic(rename = "context")]
    Context,
    #[cynic(rename = "context__id")]
    ContextId,
    #[cynic(rename = "context__timestamp")]
    ContextTimestamp,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
