use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVault {
    #[serde(alias = "chainId")]
    pub chain_id: u32,
    #[serde(alias = "vaultId")]
    pub vault_id: String,
    pub token: String,
    pub owner: String,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
    #[serde(alias = "tokenName")]
    pub token_name: String,
    #[serde(alias = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(alias = "tokenDecimals")]
    pub token_decimals: u8,
    pub balance: String,
    #[serde(alias = "inputOrders")]
    pub input_orders: Option<String>,
    #[serde(alias = "outputOrders")]
    pub output_orders: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub owners: Vec<String>,
    pub tokens: Vec<String>,
    pub hide_zero_balance: bool,
}

const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "\nAND lower(o.owner) IN ({list})\n";

const TOKENS_CLAUSE: &str = "/*TOKENS_CLAUSE*/";
const TOKENS_CLAUSE_BODY: &str = "\nAND lower(o.token) IN ({list})\n";

const CHAIN_IDS_CLAUSE: &str = "/*CHAIN_IDS_CLAUSE*/";
const CHAIN_IDS_BODY: &str = "AND vd.chain_id IN ({list})";

const ORDERBOOKS_CLAUSE: &str = "/*ORDERBOOKS_CLAUSE*/";
const ORDERBOOKS_BODY: &str = "AND lower(vd.orderbook_address) IN ({list})";

const HIDE_ZERO_BALANCE_CLAUSE: &str = "/*HIDE_ZERO_BALANCE*/";
const HIDE_ZERO_BALANCE_BODY: &str = r##"
AND NOT FLOAT_IS_ZERO(
    COALESCE((
      SELECT FLOAT_SUM(vd.delta)
      FROM vault_deltas vd
      WHERE vd.chain_id = o.chain_id
        AND lower(vd.orderbook_address) = lower(o.orderbook_address)
        AND lower(vd.owner)    = lower(o.owner)
        AND lower(vd.token)    = lower(o.token)
        AND lower(vd.vault_id) = lower(o.vault_id)
    ), FLOAT_ZERO_HEX())
  )
"##;

const INNER_CHAIN_IDS_CLAUSE: &str = "/*INNER_CHAIN_IDS_CLAUSE*/";
const INNER_CHAIN_IDS_BODY: &str = "AND chain_id IN ({list})";
const INNER_ORDERBOOKS_CLAUSE: &str = "/*INNER_ORDERBOOKS_CLAUSE*/";
const INNER_ORDERBOOKS_BODY: &str = "AND lower(orderbook_address) IN ({list})";

pub fn build_fetch_vaults_stmt(args: &FetchVaultsArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    // Chain ids (sorted/deduped)
    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();
    let chain_ids_iter = || chain_ids.iter().copied().map(|id| SqlValue::U64(id as u64));
    stmt.bind_list_clause(CHAIN_IDS_CLAUSE, CHAIN_IDS_BODY, chain_ids_iter())?;
    stmt.bind_list_clause(
        INNER_CHAIN_IDS_CLAUSE,
        INNER_CHAIN_IDS_BODY,
        chain_ids_iter(),
    )?;

    // Orderbooks (lowercase, deduped)
    let mut orderbooks = args.orderbook_addresses.clone();
    orderbooks.sort();
    orderbooks.dedup();
    let orderbooks_iter = || {
        orderbooks
            .iter()
            .map(|addr| SqlValue::Text(addr.to_string().to_ascii_lowercase()))
    };
    stmt.bind_list_clause(ORDERBOOKS_CLAUSE, ORDERBOOKS_BODY, orderbooks_iter())?;
    stmt.bind_list_clause(
        INNER_ORDERBOOKS_CLAUSE,
        INNER_ORDERBOOKS_BODY,
        orderbooks_iter(),
    )?;

    // Owners list (trim, non-empty, lowercase) with order-preserving dedup
    let mut owners: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for owner in args.owners.iter() {
        let t = owner.trim();
        if t.is_empty() {
            continue;
        }
        let lowered = t.to_ascii_lowercase();
        if seen.insert(lowered.clone()) {
            owners.push(lowered);
        }
    }
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners.into_iter().map(SqlValue::Text),
    )?;

    // Tokens list (trim, non-empty, lowercase) with order-preserving dedup
    let mut tokens: Vec<String> = Vec::new();
    let mut seen_tokens: HashSet<String> = HashSet::new();
    for token in args.tokens.iter() {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        let lowered = t.to_ascii_lowercase();
        if seen_tokens.insert(lowered.clone()) {
            tokens.push(lowered);
        }
    }
    stmt.bind_list_clause(
        TOKENS_CLAUSE,
        TOKENS_CLAUSE_BODY,
        tokens.into_iter().map(SqlValue::Text),
    )?;

    // Hide zero balance clause
    if args.hide_zero_balance {
        stmt.replace(HIDE_ZERO_BALANCE_CLAUSE, HIDE_ZERO_BALANCE_BODY)?;
    } else {
        stmt.replace(HIDE_ZERO_BALANCE_CLAUSE, "")?;
    }

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;
    use std::str::FromStr;

    use super::*;

    fn mk_args() -> FetchVaultsArgs {
        FetchVaultsArgs::default()
    }

    #[test]
    fn chain_id_and_no_filters() {
        let args = mk_args();
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("ORDER BY o.chain_id"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(stmt.params.is_empty());
    }

    #[test]
    fn owners_tokens_and_hide_zero() {
        let mut args = mk_args();
        args.owners = vec![" 0xA ".into(), "O'Owner".into()];
        args.tokens = vec!["TOK'A".into()];
        args.hide_zero_balance = true;
        args.chain_ids = vec![137, 1, 137];
        args.orderbook_addresses = vec![
            Address::from_str("0xabc0000000000000000000000000000000000000").unwrap(),
            Address::from_str("0xdef0000000000000000000000000000000000000").unwrap(),
        ];
        let stmt = build_fetch_vaults_stmt(&args).unwrap();

        // Clauses inserted
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        // Params include chain ids, orderbooks, owners, and tokens
        assert!(!stmt.params.is_empty());
    }

    #[test]
    fn missing_hide_zero_marker_yields_error() {
        // Remove the HIDE_ZERO_BALANCE marker to simulate template drift.
        let bad_template = QUERY_TEMPLATE.replace(HIDE_ZERO_BALANCE_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // replace should error because the marker is absent
        let err = stmt
            .replace(HIDE_ZERO_BALANCE_CLAUSE, HIDE_ZERO_BALANCE_BODY)
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }

    #[test]
    fn hide_zero_clause_without_filters_has_no_placeholders() {
        let mut args = mk_args();
        args.hide_zero_balance = true;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.params.is_empty());
        assert!(!stmt.sql.contains("?1"));
        assert!(!stmt.sql.contains("?2"));
    }
}
