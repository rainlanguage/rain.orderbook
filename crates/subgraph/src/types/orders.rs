use cynic::{coercions::CoercesTo, schema::NamedType};

use self::schema::variable::{self, Variable};
#[cynic::schema("orders")]
pub mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct OrdersQueryWhere {
    pub active: Option<bool>,
}

impl CoercesTo<schema::Order_filter> for OrdersQueryWhere {}

impl variable::Variable for Option<OrdersQueryWhere> {
    const TYPE: cynic::variables::VariableType = ;
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct OrdersQueryVariables {
    pub where_filter: Option<OrdersQueryWhere>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "OrdersQueryVariables")]
pub struct OrdersQuery {
    #[arguments(where: $where_filter)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Order {
    pub id: cynic::Id,
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

#[derive(cynic::QueryFragment, Debug, Clone)]
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
    pub id: Bytes,
    pub symbol: String,
    pub decimals: i32,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
