use alloy::primitives::Address;

use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use std::collections::HashSet;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub owners: Vec<String>,
    pub tokens: Vec<String>,
    pub hide_zero_balance: bool,
}

const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "\nAND lower(o.owner) IN ({list})\n";

const TOKENS_CLAUSE: &str = "/*TOKENS_CLAUSE*/";
const TOKENS_CLAUSE_BODY: &str = "\nAND lower(o.token) IN ({list})\n";

const HIDE_ZERO_BALANCE_CLAUSE: &str = "/*HIDE_ZERO_BALANCE*/";
const HIDE_ZERO_BALANCE_BODY: &str = r##"
AND NOT FLOAT_IS_ZERO(
    COALESCE((
      SELECT FLOAT_SUM(vd.delta)
      FROM vault_deltas vd
      WHERE vd.chain_id = ?1
        AND lower(vd.orderbook_address) = lower(?2)
        AND lower(vd.owner)    = lower(o.owner)
        AND lower(vd.token)    = lower(o.token)
        AND lower(vd.vault_id) = lower(o.vault_id)
    ), FLOAT_ZERO_HEX())
  )
"##;

pub fn build_fetch_vaults_stmt(
    chain_id: u32,
    orderbook_address: Address,
    args: &FetchVaultsArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    // ?1: chain id
    stmt.push(SqlValue::U64(chain_id as u64));
    // ?2: orderbook address
    stmt.push(SqlValue::Text(orderbook_address.to_string()));

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
    use super::*;

    fn mk_args() -> FetchVaultsArgs {
        FetchVaultsArgs::default()
    }

    #[test]
    fn chain_id_and_no_filters() {
        let args = mk_args();
        let stmt = build_fetch_vaults_stmt(1, Address::ZERO, &args).unwrap();
        assert!(stmt.sql.contains("et.chain_id = ?1"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert_eq!(stmt.params.len(), 2);
    }

    #[test]
    fn owners_tokens_and_hide_zero() {
        let mut args = mk_args();
        args.owners = vec![" 0xA ".into(), "O'Owner".into()];
        args.tokens = vec!["TOK'A".into()];
        args.hide_zero_balance = true;
        let stmt = build_fetch_vaults_stmt(137, Address::ZERO, &args).unwrap();

        // Clauses inserted
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        // Params: chain id + orderbook + owners + tokens
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
}
