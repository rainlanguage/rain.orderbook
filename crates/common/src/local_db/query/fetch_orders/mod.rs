use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::utils::serde::bool_from_int_or_bool;
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersActiveFilter {
    #[default]
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Default)]
pub struct FetchOrdersArgs {
    pub filter: FetchOrdersActiveFilter,
    pub owners: Vec<String>,
    pub order_hash: Option<String>,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrder {
    #[serde(alias = "orderHash")]
    pub order_hash: String,
    pub owner: String,
    #[serde(alias = "blockTimestamp")]
    pub block_timestamp: u64,
    #[serde(alias = "blockNumber")]
    pub block_number: u64,
    #[serde(alias = "orderbookAddress")]
    pub orderbook_address: String,
    #[serde(alias = "orderBytes")]
    pub order_bytes: String,
    #[serde(alias = "transactionHash")]
    pub transaction_hash: String,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    #[serde(alias = "tradeCount")]
    pub trade_count: u64,
    #[serde(deserialize_with = "bool_from_int_or_bool")]
    pub active: bool,
    pub meta: Option<String>,
}

/// Builds the SQL query fetching orders from the local database based on the
/// supplied filters.
const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "AND lower(l.order_owner) IN ({list})";

const ORDER_HASH_CLAUSE: &str = "/*ORDER_HASH_CLAUSE*/";
const ORDER_HASH_CLAUSE_BODY: &str =
    "AND lower(COALESCE(la.order_hash, l.order_hash)) = lower({param})";

const TOKENS_CLAUSE: &str = "/*TOKENS_CLAUSE*/";
const TOKENS_CLAUSE_BODY: &str =
    "AND EXISTS ( \n      SELECT 1 FROM order_ios io2 \n      WHERE io2.transaction_hash = la.transaction_hash \n        AND io2.log_index = la.log_index \n        AND lower(io2.token) IN ({list}) )";

pub fn build_fetch_orders_stmt(args: &FetchOrdersArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    // ?1 active filter
    let active_str = match args.filter {
        FetchOrdersActiveFilter::All => "all",
        FetchOrdersActiveFilter::Active => "active",
        FetchOrdersActiveFilter::Inactive => "inactive",
    };
    stmt.push(SqlValue::Text(active_str.to_string()));

    // Owners list (lowercased, trimmed, non-empty)
    let owners_lower = args
        .owners
        .iter()
        .filter_map(|o| {
            let t = o.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_ascii_lowercase())
            }
        })
        .collect::<Vec<_>>();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners_lower.into_iter().map(SqlValue::Text),
    )?;

    // Optional order hash param
    let order_hash_val = args
        .order_hash
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| SqlValue::Text(s.to_string()));
    stmt.bind_param_clause(ORDER_HASH_CLAUSE, ORDER_HASH_CLAUSE_BODY, order_hash_val)?;

    // Tokens list
    let tokens_lower = args
        .tokens
        .iter()
        .filter_map(|t| {
            let x = t.trim();
            if x.is_empty() {
                None
            } else {
                Some(x.to_ascii_lowercase())
            }
        })
        .collect::<Vec<_>>();
    stmt.bind_list_clause(
        TOKENS_CLAUSE,
        TOKENS_CLAUSE_BODY,
        tokens_lower.into_iter().map(SqlValue::Text),
    )?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_args() -> FetchOrdersArgs {
        FetchOrdersArgs::default()
    }

    #[test]
    fn filter_active_all_and_no_extras() {
        let mut args = mk_args();
        args.filter = FetchOrdersActiveFilter::All;
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'all'"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
    }

    #[test]
    fn owners_tokens_and_order_hash_filters_with_params() {
        let mut args = mk_args();
        args.filter = FetchOrdersActiveFilter::Active;
        args.owners = vec![" 0xA ".into(), "".into(), "Owner".into()];
        args.tokens = vec!["TOKA".into(), "   ".into()];
        args.order_hash = Some(" 0xHash ".into());

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        // Active filter parameterized
        assert!(stmt.sql.contains("?1 = 'active'"));

        // Owners clause present, tokens clause present, order hash clause present
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));

        // Params include at least one for the active filter
        assert!(!stmt.params.is_empty());
    }

    #[test]
    fn filter_inactive_string() {
        let mut args = mk_args();
        args.filter = FetchOrdersActiveFilter::Inactive;
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'inactive'"));
    }

    #[test]
    fn missing_order_hash_marker_yields_error() {
        // Simulate the ORDER_HASH_CLAUSE marker being removed from the template.
        let bad_template = QUERY_TEMPLATE.replace(ORDER_HASH_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // Push the active filter param that the template expects as ?1
        stmt.push(SqlValue::Text("all".to_string()));
        let err = stmt
            .bind_param_clause(
                ORDER_HASH_CLAUSE,
                ORDER_HASH_CLAUSE_BODY,
                Some(SqlValue::Text("0xhash".into())),
            )
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }

    #[test]
    fn missing_param_token_in_body_yields_error() {
        let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
        // Push the active filter param (?1)
        stmt.push(SqlValue::Text("all".to_string()));
        // Remove {param} token from the body to simulate drift between code and template
        let bad_body = ORDER_HASH_CLAUSE_BODY.replace("{param}", "");
        let err = stmt
            .bind_param_clause(
                ORDER_HASH_CLAUSE,
                &bad_body,
                Some(SqlValue::Text("0xhash".into())),
            )
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }
}
