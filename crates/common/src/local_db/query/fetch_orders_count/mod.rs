use super::fetch_orders::{FetchOrdersActiveFilter, FetchOrdersArgs};
use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrdersCountRow {
    pub orders_count: u32,
}

const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "AND l.order_owner IN ({list})";

const ORDER_HASH_CLAUSE: &str = "/*ORDER_HASH_CLAUSE*/";
const ORDER_HASH_CLAUSE_BODY: &str = "AND COALESCE(la.order_hash, l.order_hash) = {param}";

const INPUT_TOKENS_CLAUSE: &str = "/*INPUT_TOKENS_CLAUSE*/";
const INPUT_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.orderbook_address = l.orderbook_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND lower(io2.io_type) = 'input'
        AND io2.token IN ({list})
    )";

const OUTPUT_TOKENS_CLAUSE: &str = "/*OUTPUT_TOKENS_CLAUSE*/";
const OUTPUT_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.orderbook_address = l.orderbook_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND lower(io2.io_type) = 'output'
        AND io2.token IN ({list})
    )";

const COMBINED_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.orderbook_address = l.orderbook_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND (
          (lower(io2.io_type) = 'input' AND io2.token IN ({input_list}))
          OR
          (lower(io2.io_type) = 'output' AND io2.token IN ({output_list}))
        )
    )";

const MAIN_CHAIN_IDS_CLAUSE: &str = "/*MAIN_CHAIN_IDS_CLAUSE*/";
const MAIN_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
const MAIN_ORDERBOOKS_CLAUSE: &str = "/*MAIN_ORDERBOOKS_CLAUSE*/";
const MAIN_ORDERBOOKS_CLAUSE_BODY: &str = "AND oe.orderbook_address IN ({list})";

const LATEST_ADD_CHAIN_IDS_CLAUSE: &str = "/*LATEST_ADD_CHAIN_IDS_CLAUSE*/";
const LATEST_ADD_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
const LATEST_ADD_ORDERBOOKS_CLAUSE: &str = "/*LATEST_ADD_ORDERBOOKS_CLAUSE*/";
const LATEST_ADD_ORDERBOOKS_CLAUSE_BODY: &str = "AND oe.orderbook_address IN ({list})";

pub fn build_fetch_orders_count_stmt(
    args: &FetchOrdersArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    let active_str = match args.filter {
        FetchOrdersActiveFilter::All => "all",
        FetchOrdersActiveFilter::Active => "active",
        FetchOrdersActiveFilter::Inactive => "inactive",
    };
    stmt.push(SqlValue::from(active_str));

    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut orderbooks = args.orderbook_addresses.clone();
    orderbooks.sort();
    orderbooks.dedup();

    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let orderbooks_iter = || orderbooks.iter().cloned().map(SqlValue::from);

    stmt.bind_list_clause(
        MAIN_CHAIN_IDS_CLAUSE,
        MAIN_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        LATEST_ADD_CHAIN_IDS_CLAUSE,
        LATEST_ADD_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;

    stmt.bind_list_clause(
        MAIN_ORDERBOOKS_CLAUSE,
        MAIN_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        LATEST_ADD_ORDERBOOKS_CLAUSE,
        LATEST_ADD_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;

    let mut owners = args.owners.clone();
    owners.sort();
    owners.dedup();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners.into_iter().map(SqlValue::from),
    )?;

    let order_hash_val = args.order_hash.as_ref().map(|hash| SqlValue::from(*hash));
    stmt.bind_param_clause(ORDER_HASH_CLAUSE, ORDER_HASH_CLAUSE_BODY, order_hash_val)?;

    let mut input_tokens = args.tokens.inputs.clone();
    input_tokens.sort();
    input_tokens.dedup();

    let mut output_tokens = args.tokens.outputs.clone();
    output_tokens.sort();
    output_tokens.dedup();

    let has_inputs = !input_tokens.is_empty();
    let has_outputs = !output_tokens.is_empty();

    if has_inputs && has_outputs && input_tokens == output_tokens {
        let input_placeholders: Vec<String> = input_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let input_list_str = input_placeholders.join(", ");

        for token in &input_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let output_placeholders: Vec<String> = output_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let output_list_str = output_placeholders.join(", ");

        for token in &output_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let combined_clause = COMBINED_TOKENS_CLAUSE_BODY
            .replace("{input_list}", &input_list_str)
            .replace("{output_list}", &output_list_str);

        stmt.sql = stmt.sql.replace(INPUT_TOKENS_CLAUSE, &combined_clause);
        stmt.sql = stmt.sql.replace(OUTPUT_TOKENS_CLAUSE, "");
    } else {
        stmt.bind_list_clause(
            INPUT_TOKENS_CLAUSE,
            INPUT_TOKENS_CLAUSE_BODY,
            input_tokens.into_iter().map(SqlValue::from),
        )?;
        stmt.bind_list_clause(
            OUTPUT_TOKENS_CLAUSE,
            OUTPUT_TOKENS_CLAUSE_BODY,
            output_tokens.into_iter().map(SqlValue::from),
        )?;
    }

    Ok(stmt)
}

pub fn extract_orders_count(rows: &[LocalDbOrdersCountRow]) -> u32 {
    rows.first().map(|row| row.orders_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256, Address};

    #[test]
    fn builds_count_with_no_filters() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("COUNT(*)"));
        assert!(stmt.sql.contains("orders_count"));
        assert!(stmt.sql.contains("?1 = 'all'"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
    }

    #[test]
    fn builds_count_with_active_filter() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137],
            filter: FetchOrdersActiveFilter::Active,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'active'"));
    }

    #[test]
    fn builds_count_with_owners_and_order_hash() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            owners: vec![address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB")],
            order_hash: Some(b256!(
                "0x00000000000000000000000000000000000000000000000000000000deadbeef"
            )),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("l.order_owner IN ("));
        assert!(stmt
            .sql
            .contains("COALESCE(la.order_hash, l.order_hash) = "));
    }

    #[test]
    fn builds_count_with_chain_ids() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1, 137],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(!stmt.sql.contains(MAIN_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(LATEST_ADD_CHAIN_IDS_CLAUSE));
        assert!(stmt.sql.contains("AND oe.chain_id IN (?"));
    }

    #[test]
    fn builds_count_with_orderbooks() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            orderbook_addresses: vec![Address::ZERO],
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND oe.orderbook_address IN (?"));
    }

    #[test]
    fn builds_count_with_combined_tokens_when_inputs_equal_outputs() {
        use super::super::fetch_orders::FetchOrdersTokensFilter;

        let token_a = address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB");
        let token_b = address!("0x1111111111111111111111111111111111111111");
        let tokens = vec![token_a, token_b];

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: tokens.clone(),
                outputs: tokens,
            },
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();

        assert!(
            stmt.sql.contains("lower(io2.io_type) = 'input'"),
            "should contain input check in combined clause"
        );
        assert!(
            stmt.sql.contains("lower(io2.io_type) = 'output'"),
            "should contain output check in combined clause"
        );
        assert!(
            stmt.sql.contains(" OR "),
            "should use OR to combine input and output checks"
        );
        assert!(
            !stmt.sql.contains(INPUT_TOKENS_CLAUSE),
            "input tokens placeholder should be replaced"
        );
        assert!(
            !stmt.sql.contains(OUTPUT_TOKENS_CLAUSE),
            "output tokens placeholder should be replaced"
        );
    }

    #[test]
    fn builds_count_with_separate_tokens_when_inputs_differ_from_outputs() {
        use super::super::fetch_orders::FetchOrdersTokensFilter;

        let token_a = address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB");
        let token_b = address!("0x1111111111111111111111111111111111111111");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![token_a],
                outputs: vec![token_b],
            },
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();

        assert!(
            !stmt.sql.contains(INPUT_TOKENS_CLAUSE),
            "input tokens placeholder should be replaced"
        );
        assert!(
            !stmt.sql.contains(OUTPUT_TOKENS_CLAUSE),
            "output tokens placeholder should be replaced"
        );
        let exists_count = stmt.sql.matches("AND EXISTS").count();
        assert_eq!(
            exists_count, 2,
            "should have two separate EXISTS subqueries when tokens differ"
        );
    }

    #[test]
    fn extract_count_returns_value() {
        let rows = vec![LocalDbOrdersCountRow { orders_count: 42 }];
        assert_eq!(extract_orders_count(&rows), 42);
    }

    #[test]
    fn extract_count_returns_zero_for_empty() {
        let rows: Vec<LocalDbOrdersCountRow> = vec![];
        assert_eq!(extract_orders_count(&rows), 0);
    }
}
