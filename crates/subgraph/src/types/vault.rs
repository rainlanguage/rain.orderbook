use crate::schema;
use typeshare::typeshare;

#[typeshare]
#[derive(cynic::QueryVariables, Debug)]
pub struct VaultQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "VaultQueryVariables")]
pub struct VaultQuery {
    #[arguments(id: $id)]
    pub vault: Option<Vault>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub id: cynic::Id,
    pub owner: Account,
    pub deposits: Option<Vec<VaultDeposit>>,
    pub withdraws: Option<Vec<VaultWithdraw>>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct VaultWithdraw {
    pub id: cynic::Id,
    pub sender: Account,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub requested_amount: BigInt,
    pub requested_amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub sender: Account,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: Bytes,
}

#[typeshare]
#[serde(rename = "BigDecimalString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[serde(rename = "BigIntString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[serde(rename = "BytesString")]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
