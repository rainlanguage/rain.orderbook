use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::utils::serde::bool_from_int_or_bool;
use alloy::primitives::{Address, Bytes, B256};
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
pub struct FetchOrdersTokensFilter {
    pub inputs: Vec<Address>,
    pub outputs: Vec<Address>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchOrdersArgs {
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub filter: FetchOrdersActiveFilter,
    pub owners: Vec<Address>,
    pub order_hash: Option<B256>,
    pub tx_hash: Option<B256>,
    pub tokens: FetchOrdersTokensFilter,
    pub page: Option<u16>,
    pub page_size: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbOrder {
    pub chain_id: u32,
    pub order_hash: B256,
    pub owner: Address,
    pub block_timestamp: u64,
    pub block_number: u64,
    pub orderbook_address: Address,
    pub order_bytes: Bytes,
    pub transaction_hash: B256,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    pub trade_count: u64,
    #[serde(deserialize_with = "bool_from_int_or_bool")]
    pub active: bool,
    pub meta: Option<Bytes>,
}

/// Builds the SQL query fetching orders from the local database based on the
/// supplied filters.
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

const FIRST_ADD_CHAIN_IDS_CLAUSE: &str = "/*FIRST_ADD_CHAIN_IDS_CLAUSE*/";
const FIRST_ADD_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
const FIRST_ADD_ORDERBOOKS_CLAUSE: &str = "/*FIRST_ADD_ORDERBOOKS_CLAUSE*/";
const FIRST_ADD_ORDERBOOKS_CLAUSE_BODY: &str = "AND oe.orderbook_address IN ({list})";

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE: &str = "/*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY: &str = "AND t.orderbook_address IN ({list})";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND entries.chain_id IN ({list})";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY: &str = "AND entries.orderbook_address IN ({list})";
const TX_HASH_CLAUSE: &str = "/*TX_HASH_CLAUSE*/";
const TX_HASH_CLAUSE_BODY: &str = "AND oe.transaction_hash = {param}";
const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";

pub fn build_fetch_orders_stmt(args: &FetchOrdersArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    // ?1 active filter
    let active_str = match args.filter {
        FetchOrdersActiveFilter::All => "all",
        FetchOrdersActiveFilter::Active => "active",
        FetchOrdersActiveFilter::Inactive => "inactive",
    };
    stmt.push(SqlValue::from(active_str));

    // Chain ids (deduplicated, sorted)
    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    // Orderbook addresses (lowercase, deduplicated)
    let mut orderbooks = args.orderbook_addresses.clone();
    orderbooks.sort();
    orderbooks.dedup();

    // Helper closures to bind repeated clauses without ownership issues
    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let orderbooks_iter = || orderbooks.iter().cloned().map(SqlValue::from);

    // Apply chain-id filters across query sections
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
        FIRST_ADD_CHAIN_IDS_CLAUSE,
        FIRST_ADD_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_ORDERS_CHAIN_IDS_CLAUSE,
        TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE,
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;

    // Apply orderbook filters if provided
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
    stmt.bind_list_clause(
        FIRST_ADD_ORDERBOOKS_CLAUSE,
        FIRST_ADD_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_ORDERS_ORDERBOOKS_CLAUSE,
        TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE,
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;

    // Optional tx hash param
    let tx_hash_val = args.tx_hash.as_ref().map(|hash| SqlValue::from(*hash));
    stmt.bind_param_clause(TX_HASH_CLAUSE, TX_HASH_CLAUSE_BODY, tx_hash_val)?;

    let mut owners_lower = args.owners.clone();
    owners_lower.sort();
    owners_lower.dedup();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners_lower.into_iter().map(SqlValue::from),
    )?;

    // Optional order hash param
    let order_hash_val = args.order_hash.as_ref().map(|hash| SqlValue::from(*hash));
    stmt.bind_param_clause(ORDER_HASH_CLAUSE, ORDER_HASH_CLAUSE_BODY, order_hash_val)?;

    // Directional token filters
    // - When inputs == outputs (identical lists): OR logic for "any-IO" filtering
    // - When inputs != outputs (different lists): AND logic for directional filtering
    let mut input_tokens = args.tokens.inputs.clone();
    input_tokens.sort();
    input_tokens.dedup();

    let mut output_tokens = args.tokens.outputs.clone();
    output_tokens.sort();
    output_tokens.dedup();

    let has_inputs = !input_tokens.is_empty();
    let has_outputs = !output_tokens.is_empty();

    if has_inputs && has_outputs && input_tokens == output_tokens {
        // Combined OR clause: matches orders where token is in inputs OR outputs
        // Used when lists are identical for "any-IO" behavior (e.g., UI token filter)

        // Build parameter placeholders for input tokens
        let input_placeholders: Vec<String> = input_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let input_list_str = input_placeholders.join(", ");

        // Push input token params
        for token in &input_tokens {
            stmt.push(SqlValue::from(*token));
        }

        // Build parameter placeholders for output tokens
        let output_placeholders: Vec<String> = output_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let output_list_str = output_placeholders.join(", ");

        // Push output token params
        for token in &output_tokens {
            stmt.push(SqlValue::from(*token));
        }

        // Build the combined clause
        let combined_clause = COMBINED_TOKENS_CLAUSE_BODY
            .replace("{input_list}", &input_list_str)
            .replace("{output_list}", &output_list_str);

        // Replace INPUT_TOKENS_CLAUSE with combined clause, clear OUTPUT_TOKENS_CLAUSE
        stmt.sql = stmt.sql.replace(INPUT_TOKENS_CLAUSE, &combined_clause);
        stmt.sql = stmt.sql.replace(OUTPUT_TOKENS_CLAUSE, "");
    } else {
        // Separate EXISTS clauses with AND logic:
        // - When only inputs or only outputs specified: single-direction filtering
        // - When both specified but different: directional filtering (input AND output)
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

    if let (Some(page), Some(page_size)) = (args.page, args.page_size) {
        let offset = (page.saturating_sub(1) as u64) * (page_size as u64);
        let limit_placeholder = format!("?{}", stmt.params.len() + 1);
        let offset_placeholder = format!("?{}", stmt.params.len() + 2);
        let pagination = format!("LIMIT {} OFFSET {}", limit_placeholder, offset_placeholder);
        stmt.sql = stmt.sql.replace(PAGINATION_CLAUSE, &pagination);
        stmt.push(SqlValue::U64(page_size as u64));
        stmt.push(SqlValue::U64(offset));
    } else {
        stmt.sql = stmt.sql.replace(PAGINATION_CLAUSE, "");
    }

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::hex;
    use alloy::primitives::{address, b256};
    use std::str::FromStr;

    #[test]
    fn filter_active_all_and_no_extras() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'all'"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
    }

    #[test]
    fn owners_tokens_and_order_hash_filters_with_params() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1],
            orderbook_addresses: vec![address!("0xabcdef0000000000000000000000000000000000")],
            filter: FetchOrdersActiveFilter::Active,
            owners: vec![
                address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB"),
                address!("0x7D3Dd01feD0C16A6c353ce3BACF26467726EF96e"),
                address!("0x87d08841bdAd4aB82883a322D2c0eF557EC154fE"),
            ],
            order_hash: Some(b256!(
                "0x00000000000000000000000000000000000000000000000000000000deadbeef"
            )),
            tx_hash: None,
            tokens: FetchOrdersTokensFilter {
                inputs: vec![
                    address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB"),
                    address!("0x7D3Dd01feD0C16A6c353ce3BACF26467726EF96e"),
                ],
                outputs: vec![address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB")],
            },
            page: None,
            page_size: None,
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        // Active filter parameterized
        assert!(stmt.sql.contains("?1 = 'active'"));

        // Owners clause present, tokens clause present, order hash clause present
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));

        // Params include active filter followed by chain/orderbook filters
        assert!(stmt.params.len() >= 3);
        assert_eq!(stmt.params[0], SqlValue::Text("active".to_string()));
    }

    #[test]
    fn input_tokens_clause_only_when_inputs_present() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![address!("0x00000000000000000000000000000000000000aa")],
                outputs: vec![],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
        assert!(stmt.sql.contains("AND io2.token IN ("));
        assert!(!stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
    }

    #[test]
    fn output_tokens_clause_only_when_outputs_present() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![],
                outputs: vec![address!("0x00000000000000000000000000000000000000bb")],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
        assert!(stmt.sql.contains("AND io2.token IN ("));
        assert!(!stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
    }

    #[test]
    fn combined_or_clause_when_inputs_and_outputs_identical() {
        // When inputs == outputs, use OR logic for "any-IO" filtering
        let token_addr = address!("0x00000000000000000000000000000000000000aa");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![token_addr],
                outputs: vec![token_addr],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        // Should have a single EXISTS with OR logic inside
        assert!(stmt
            .sql
            .contains("(lower(io2.io_type) = 'input' AND io2.token IN ("));
        assert!(stmt
            .sql
            .contains("(lower(io2.io_type) = 'output' AND io2.token IN ("));
        assert!(stmt.sql.contains(" OR "));
        // Should only have one EXISTS clause, not two
        assert_eq!(stmt.sql.matches("AND EXISTS (").count(), 1);
    }

    #[test]
    fn separate_and_clauses_when_inputs_and_outputs_differ() {
        // When inputs != outputs, use AND logic for directional filtering
        let input_addr = address!("0x00000000000000000000000000000000000000aa");
        let output_addr = address!("0x00000000000000000000000000000000000000bb");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![input_addr],
                outputs: vec![output_addr],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        // Should have two separate EXISTS clauses (AND logic)
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
        // Should NOT have OR in the token filtering (io_type check)
        assert!(!stmt.sql.contains("(lower(io2.io_type) = 'input'"));
        // Should have two EXISTS clauses
        assert_eq!(stmt.sql.matches("AND EXISTS (").count(), 2);

        // Verify both tokens are in params
        let input_param = SqlValue::from(input_addr);
        let output_param = SqlValue::from(output_addr);
        assert!(stmt.params.contains(&input_param));
        assert!(stmt.params.contains(&output_param));
    }

    #[test]
    fn filter_inactive_string() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::Inactive,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'inactive'"));
    }

    #[test]
    fn chain_id_filters_apply_across_all_sections() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1, 137],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        for marker in [
            MAIN_CHAIN_IDS_CLAUSE,
            LATEST_ADD_CHAIN_IDS_CLAUSE,
            FIRST_ADD_CHAIN_IDS_CLAUSE,
            TAKE_ORDERS_CHAIN_IDS_CLAUSE,
            CLEAR_EVENTS_CHAIN_IDS_CLAUSE,
        ] {
            assert!(
                !stmt.sql.contains(marker),
                "expected marker {marker} to be replaced"
            );
        }

        assert!(
            stmt.sql.contains("AND oe.chain_id IN (?"),
            "main chain filter missing"
        );
        assert!(
            stmt.sql.contains("AND t.chain_id IN (?"),
            "take_orders chain filter missing"
        );
        assert!(
            stmt.sql.contains("AND entries.chain_id IN (?"),
            "clear_events chain filter missing"
        );

        // Chain IDs should be deduplicated before binding; only 2 unique values expected.
        let unique_chain_ids: std::collections::HashSet<_> = stmt
            .params
            .iter()
            .filter_map(|value| match value {
                SqlValue::U64(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert!(
            unique_chain_ids.contains(&1_u64) && unique_chain_ids.contains(&137_u64),
            "expected both chain IDs to be present in params"
        );
    }

    #[test]
    fn orderbook_filters_lowercase_and_optional() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            orderbook_addresses: vec![
                Address::from_str("0xAbCDeF0000000000000000000000000000000000").unwrap(),
                Address::from_str("0xabcdef0000000000000000000000000000000000").unwrap(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            ],
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        for marker in [
            MAIN_ORDERBOOKS_CLAUSE,
            LATEST_ADD_ORDERBOOKS_CLAUSE,
            FIRST_ADD_ORDERBOOKS_CLAUSE,
            TAKE_ORDERS_ORDERBOOKS_CLAUSE,
            CLEAR_EVENTS_ORDERBOOKS_CLAUSE,
        ] {
            assert!(
                !stmt.sql.contains(marker),
                "expected marker {marker} to be replaced"
            );
        }

        assert!(
            stmt.sql.contains("AND oe.orderbook_address IN (?"),
            "main orderbook filter missing"
        );
        assert!(
            stmt.sql.contains("AND t.orderbook_address IN (?"),
            "take_orders orderbook filter missing"
        );

        // Only the trimmed, lowercased address should appear in bound params.
        let lower_addr = "0xabcdef0000000000000000000000000000000000";
        let text_params: Vec<&String> = stmt
            .params
            .iter()
            .filter_map(|value| match value {
                SqlValue::Text(text) => Some(text),
                _ => None,
            })
            .collect();
        assert!(
            text_params.iter().any(|text| text.as_str() == lower_addr),
            "expected lowercase orderbook address in params"
        );
        for text in text_params {
            let lowered = text.to_ascii_lowercase();
            assert_eq!(
                text.as_str(),
                lowered.as_str(),
                "orderbook param should be lowercase"
            );
        }

        // When orderbooks are omitted entirely, no orderbook clause should remain.
        let args_no_orderbooks = FetchOrdersArgs {
            chain_ids: vec![1],
            ..FetchOrdersArgs::default()
        };
        let stmt_no_orderbooks = build_fetch_orders_stmt(&args_no_orderbooks).unwrap();
        assert!(
            !stmt_no_orderbooks.sql.contains("oe.orderbook_address IN ("),
            "orderbook clause should not appear when list is empty"
        );
    }

    #[test]
    fn tx_hash_clause_included_when_present() {
        let tx_hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tx_hash: Some(tx_hash),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(
            !stmt.sql.contains(TX_HASH_CLAUSE),
            "tx hash marker should be replaced"
        );
        assert!(
            stmt.sql.contains("oe.transaction_hash = ?"),
            "tx hash clause should be present"
        );
        let expected = SqlValue::Text(hex::encode_prefixed(tx_hash));
        assert!(
            stmt.params.contains(&expected),
            "tx hash param should be bound"
        );
    }

    #[test]
    fn pagination_clause_page1() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(1),
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"), "should contain LIMIT clause");
        assert!(stmt.sql.contains("OFFSET"), "should contain OFFSET clause");
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(10));
        assert_eq!(last_two[0], &SqlValue::U64(0));
    }

    #[test]
    fn pagination_clause_page3() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(3),
            page_size: Some(25),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        assert!(stmt.sql.contains("OFFSET"));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(25));
        assert_eq!(last_two[0], &SqlValue::U64(50));
    }

    #[test]
    fn pagination_clause_page0_saturates_to_zero_offset() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(0),
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(10));
        assert_eq!(last_two[0], &SqlValue::U64(0));
    }

    #[test]
    fn pagination_clause_omitted_when_only_page_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(2),
            page_size: None,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn pagination_clause_omitted_when_only_page_size_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: None,
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn pagination_clause_omitted_when_neither_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn missing_order_hash_marker_yields_error() {
        // Simulate the ORDER_HASH_CLAUSE marker being removed from the template.
        let bad_template = QUERY_TEMPLATE.replace(ORDER_HASH_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // Push the fixed params expected by the template (?1 active filter)
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
        // Push the fixed params (?1 active filter)
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
