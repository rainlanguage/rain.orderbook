use alloy::primitives::Address;

use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub owners: Vec<Address>,
    pub tokens: Vec<Address>,
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
    ob_id: &OrderbookIdentifier,
    args: &FetchVaultsArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));

    let mut owners = args.owners.clone();
    owners.sort();
    owners.dedup();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners.into_iter().map(SqlValue::from),
    )?;

    let mut tokens = args.tokens.clone();
    tokens.sort();
    tokens.dedup();
    stmt.bind_list_clause(
        TOKENS_CLAUSE,
        TOKENS_CLAUSE_BODY,
        tokens.into_iter().map(SqlValue::from),
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
    use alloy::primitives::{address, Address};

    use super::*;

    fn mk_args() -> FetchVaultsArgs {
        FetchVaultsArgs::default()
    }

    #[test]
    fn chain_id_and_no_filters() {
        let args = mk_args();
        let stmt =
            build_fetch_vaults_stmt(&OrderbookIdentifier::new(1, Address::ZERO), &args).unwrap();
        assert!(stmt.sql.contains("et.chain_id = ?1"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert_eq!(stmt.params.len(), 2);
    }

    #[test]
    fn owners_tokens_and_hide_zero() {
        let mut args = mk_args();
        args.owners = vec![
            address!("0x87d08841bdAd4aB82883a322D2c0eF557EC154fE"),
            address!("0x632ffCd874c1dDD5aCf9c26918D31CA3c96c0ec8"),
        ];
        args.tokens = vec![address!("0x1AC6F2786A51b20d47050f3f9E4B0e831427B498")];
        args.hide_zero_balance = true;
        let stmt =
            build_fetch_vaults_stmt(&OrderbookIdentifier::new(137, Address::ZERO), &args).unwrap();

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
