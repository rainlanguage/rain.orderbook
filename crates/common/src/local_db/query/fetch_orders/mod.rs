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
pub fn build_fetch_orders_query(args: &FetchOrdersArgs) -> String {
    let filter_str = match args.filter {
        FetchOrdersActiveFilter::All => "all",
        FetchOrdersActiveFilter::Active => "active",
        FetchOrdersActiveFilter::Inactive => "inactive",
    };

    let sanitize_literal = |value: &str| value.replace('\'', "''");

    let owner_values: Vec<String> = args
        .owners
        .iter()
        .filter_map(|owner| {
            let trimmed = owner.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("'{}'", sanitize_literal(&trimmed.to_lowercase())))
            }
        })
        .collect();
    let filter_owners = if owner_values.is_empty() {
        String::new()
    } else {
        format!(
            "\nAND lower(l.order_owner) IN ({})\n",
            owner_values.join(", ")
        )
    };

    let filter_order_hash = args
        .order_hash
        .as_ref()
        .and_then(|hash| {
            let trimmed = hash.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!(
                    "\nAND lower(COALESCE(la.order_hash, l.order_hash)) = lower('{}')\n",
                    sanitize_literal(trimmed)
                ))
            }
        })
        .unwrap_or_default();

    let token_values: Vec<String> = args
        .tokens
        .iter()
        .filter_map(|token| {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("'{}'", sanitize_literal(&trimmed.to_lowercase())))
            }
        })
        .collect();
    let filter_tokens = if token_values.is_empty() {
        String::new()
    } else {
        format!(
            "\nAND EXISTS (\n    SELECT 1 FROM order_ios io2\n    WHERE io2.transaction_hash = la.transaction_hash\n      AND io2.log_index = la.log_index\n      AND lower(io2.token) IN ({})\n)\n",
            token_values.join(", ")
        )
    };

    QUERY_TEMPLATE
        .replace("'?filter_active'", &format!("'{}'", filter_str))
        .replace("?filter_owners", &filter_owners)
        .replace("?filter_order_hash", &filter_order_hash)
        .replace("?filter_tokens", &filter_tokens)
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
        let q = build_fetch_orders_query(&args);
        assert!(q.contains("'all'"));
        assert!(!q.contains("?filter_owners"));
        assert!(!q.contains("?filter_tokens"));
        assert!(!q.contains("?filter_order_hash"));
    }

    #[test]
    fn owners_tokens_and_order_hash_filters_with_sanitization() {
        let mut args = mk_args();
        args.filter = FetchOrdersActiveFilter::Active;
        args.owners = vec![" 0xA ".into(), "".into(), "O'Owner".into()];
        args.tokens = vec!["TOK'A".into(), "   ".into()];
        args.order_hash = Some(" 0xHash ' ".into());

        let q = build_fetch_orders_query(&args);

        // Active filter string inserted
        assert!(q.contains("'active'"));

        // Owners filter inserted as IN clause with lowercase + sanitized values
        assert!(q.contains("AND lower(l.order_owner) IN ('0xa', 'o''owner')"));

        // Tokens filter inserted as IN clause with lowercase + sanitized values
        assert!(q.contains("lower(io2.token) IN ('tok''a')"));

        // Order hash filter present, sanitized, applied to COALESCE hash
        // Note: builder relies on SQL lower(...) rather than pre-lowercasing the literal.
        assert!(q.contains("lower(COALESCE(la.order_hash, l.order_hash)) = lower('0xHash ''')"));

        // No placeholders remain
        assert!(!q.contains("?filter_owners"));
        assert!(!q.contains("?filter_tokens"));
        assert!(!q.contains("?filter_order_hash"));
    }

    #[test]
    fn filter_inactive_string() {
        let mut args = mk_args();
        args.filter = FetchOrdersActiveFilter::Inactive;
        let q = build_fetch_orders_query(&args);
        assert!(q.contains("'inactive'"));
    }
}
