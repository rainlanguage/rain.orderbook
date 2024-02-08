use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
pub struct OrderClearsListQueryVariables {
    pub first: Option<i32>,
    pub skip: Option<i32>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
#[cynic(graphql_type = "Query", variables = "OrderClearsListQueryVariables")]
pub struct OrderClearsListQuery {
    #[arguments(orderBy: "timestamp", orderDirection: "desc", skip: $skip, first: $first)]
    pub order_clears: Vec<OrderClear>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct OrderClear {
    pub id: cynic::Id,
    pub transaction: Transaction,
    pub sender: Account,
    pub clearer: Account,
    pub timestamp: BigInt,
    pub order_a: Order,
    #[cynic(rename = "aInputIOIndex")]
    pub a_input_ioindex: BigInt,
    pub order_b: Order,
    #[cynic(rename = "bOutputIOIndex")]
    pub b_output_ioindex: BigInt,
    pub bounty: Bounty,
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
pub struct Bounty {
    pub bounty_vault_a: Vault,
    pub bounty_token_a: Erc20,
    pub bounty_amount_a: Option<BigInt>,
    #[cynic(rename = "bountyAmountADisplay")]
    pub bounty_amount_adisplay: Option<BigDecimal>,
    pub bounty_vault_b: Vault,
    pub bounty_token_b: Erc20,
    pub bounty_amount_b: Option<BigInt>,
    #[cynic(rename = "bountyAmountBDisplay")]
    pub bounty_amount_bdisplay: Option<BigDecimal>,
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
pub struct Vault {
    pub id: cynic::Id,
    pub vault_id: BigInt,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "OrderClear_orderBy")]
pub enum OrderClearOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "sender")]
    Sender,
    #[cynic(rename = "sender__id")]
    SenderId,
    #[cynic(rename = "clearer")]
    Clearer,
    #[cynic(rename = "clearer__id")]
    ClearerId,
    #[cynic(rename = "orderA")]
    OrderA,
    #[cynic(rename = "orderA__id")]
    OrderAId,
    #[cynic(rename = "orderA__orderHash")]
    OrderAOrderHash,
    #[cynic(rename = "orderA__interpreter")]
    OrderAInterpreter,
    #[cynic(rename = "orderA__interpreterStore")]
    OrderAInterpreterStore,
    #[cynic(rename = "orderA__expressionDeployer")]
    OrderAExpressionDeployer,
    #[cynic(rename = "orderA__expression")]
    OrderAExpression,
    #[cynic(rename = "orderA__orderActive")]
    OrderAOrderActive,
    #[cynic(rename = "orderA__handleIO")]
    OrderAHandleIo,
    #[cynic(rename = "orderA__orderJSONString")]
    OrderAOrderJsonstring,
    #[cynic(rename = "orderA__expressionJSONString")]
    OrderAExpressionJsonstring,
    #[cynic(rename = "orderA__timestamp")]
    OrderATimestamp,
    #[cynic(rename = "orderB")]
    OrderB,
    #[cynic(rename = "orderB__id")]
    OrderBId,
    #[cynic(rename = "orderB__orderHash")]
    OrderBOrderHash,
    #[cynic(rename = "orderB__interpreter")]
    OrderBInterpreter,
    #[cynic(rename = "orderB__interpreterStore")]
    OrderBInterpreterStore,
    #[cynic(rename = "orderB__expressionDeployer")]
    OrderBExpressionDeployer,
    #[cynic(rename = "orderB__expression")]
    OrderBExpression,
    #[cynic(rename = "orderB__orderActive")]
    OrderBOrderActive,
    #[cynic(rename = "orderB__handleIO")]
    OrderBHandleIo,
    #[cynic(rename = "orderB__orderJSONString")]
    OrderBOrderJsonstring,
    #[cynic(rename = "orderB__expressionJSONString")]
    OrderBExpressionJsonstring,
    #[cynic(rename = "orderB__timestamp")]
    OrderBTimestamp,
    #[cynic(rename = "owners")]
    Owners,
    #[cynic(rename = "aInputIOIndex")]
    AInputIoindex,
    #[cynic(rename = "aOutputIOIndex")]
    AOutputIoindex,
    #[cynic(rename = "bInputIOIndex")]
    BInputIoindex,
    #[cynic(rename = "bOutputIOIndex")]
    BOutputIoindex,
    #[cynic(rename = "bounty")]
    Bounty,
    #[cynic(rename = "bounty__id")]
    BountyId,
    #[cynic(rename = "bounty__bountyAmountA")]
    BountyBountyAmountA,
    #[cynic(rename = "bounty__bountyAmountADisplay")]
    BountyBountyAmountAdisplay,
    #[cynic(rename = "bounty__bountyAmountB")]
    BountyBountyAmountB,
    #[cynic(rename = "bounty__bountyAmountBDisplay")]
    BountyBountyAmountBdisplay,
    #[cynic(rename = "bounty__timestamp")]
    BountyTimestamp,
    #[cynic(rename = "stateChange")]
    StateChange,
    #[cynic(rename = "stateChange__id")]
    StateChangeId,
    #[cynic(rename = "stateChange__aOutput")]
    StateChangeAOutput,
    #[cynic(rename = "stateChange__bOutput")]
    StateChangeBOutput,
    #[cynic(rename = "stateChange__aInput")]
    StateChangeAInput,
    #[cynic(rename = "stateChange__bInput")]
    StateChangeBInput,
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
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);


