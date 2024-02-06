use crate::schema;
use serde::Serialize;
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct VaultBalanceChangesListQueryVariables<'a> {
    pub first: Option<i32>,
    pub id: &'a cynic::Id,
    pub skip: Option<i32>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(
    graphql_type = "Query",
    variables = "VaultBalanceChangesListQueryVariables"
)]
pub struct VaultBalanceChangesListQuery {
    #[arguments(orderBy: "timestamp", orderDirection: "desc", where: { tokenVault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_deposits: Vec<VaultDeposit>,
    #[arguments(orderBy: "timestamp", orderDirection: "desc", where: { tokenVault_: { id: $id } }, skip: $skip, first: $first)]
    pub vault_withdraws: Vec<VaultWithdraw>,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct VaultWithdraw {
    pub id: cynic::Id,
    pub vault_id: BigInt,
    pub timestamp: BigInt,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub sender: Account,
    pub transaction: Transaction,
    pub token_vault: TokenVault,
    pub requested_amount: BigInt,
    pub requested_amount_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct VaultDeposit {
    pub id: cynic::Id,
    pub vault_id: BigInt,
    pub timestamp: BigInt,
    pub amount: BigInt,
    pub amount_display: BigDecimal,
    pub sender: Account,
    pub transaction: Transaction,
    pub token_vault: TokenVault,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct TokenVault {
    pub id: cynic::Id,
    pub token: Erc20,
    pub balance: BigInt,
    pub balance_display: BigDecimal,
}

#[typeshare]
#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Transaction {
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

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderDirection {
    #[cynic(rename = "asc")]
    Asc,
    #[cynic(rename = "desc")]
    Desc,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "VaultDeposit_orderBy")]
pub enum VaultDepositOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "sender")]
    Sender,
    #[cynic(rename = "sender__id")]
    SenderId,
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
    #[cynic(rename = "vaultId")]
    VaultId,
    #[cynic(rename = "vault")]
    Vault,
    #[cynic(rename = "vault__id")]
    VaultId2,
    #[cynic(rename = "vault__vaultId")]
    VaultVaultId,
    #[cynic(rename = "amount")]
    Amount,
    #[cynic(rename = "amountDisplay")]
    AmountDisplay,
    #[cynic(rename = "tokenVault")]
    TokenVault,
    #[cynic(rename = "tokenVault__id")]
    TokenVaultId,
    #[cynic(rename = "tokenVault__vaultId")]
    TokenVaultVaultId,
    #[cynic(rename = "tokenVault__balance")]
    TokenVaultBalance,
    #[cynic(rename = "tokenVault__balanceDisplay")]
    TokenVaultBalanceDisplay,
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
#[cynic(graphql_type = "VaultWithdraw_orderBy")]
pub enum VaultWithdrawOrderBy {
    #[cynic(rename = "id")]
    Id,
    #[cynic(rename = "sender")]
    Sender,
    #[cynic(rename = "sender__id")]
    SenderId,
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
    #[cynic(rename = "vaultId")]
    VaultId,
    #[cynic(rename = "vault")]
    Vault,
    #[cynic(rename = "vault__id")]
    VaultId2,
    #[cynic(rename = "vault__vaultId")]
    VaultVaultId,
    #[cynic(rename = "requestedAmount")]
    RequestedAmount,
    #[cynic(rename = "requestedAmountDisplay")]
    RequestedAmountDisplay,
    #[cynic(rename = "amount")]
    Amount,
    #[cynic(rename = "amountDisplay")]
    AmountDisplay,
    #[cynic(rename = "tokenVault")]
    TokenVault,
    #[cynic(rename = "tokenVault__id")]
    TokenVaultId,
    #[cynic(rename = "tokenVault__vaultId")]
    TokenVaultVaultId,
    #[cynic(rename = "tokenVault__balance")]
    TokenVaultBalance,
    #[cynic(rename = "tokenVault__balanceDisplay")]
    TokenVaultBalanceDisplay,
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

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigDecimal(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct BigInt(pub String);

#[typeshare]
#[derive(cynic::Scalar, Debug, Clone)]
pub struct Bytes(pub String);
