use crate::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct VaultQueryVariables<'a> {
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "VaultQueryVariables")]
pub struct VaultQuery {
    #[arguments(id: $id)]
    pub vault: Option<Vault>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Vault {
    pub id: cynic::Id,
    pub owner: Account,
    pub deposits: Option<Vec<VaultDeposit>>,
    pub withdraws: Option<Vec<VaultWithdraw>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct VaultWithdraw {
    pub id: cynic::Id,
    pub sender: Account,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub requested_amount: BigInt,
    pub requested_amount_display: BigDecimal,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub sender: Account,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: Bytes,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
