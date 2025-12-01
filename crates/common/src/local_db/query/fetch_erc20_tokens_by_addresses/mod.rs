use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use alloy::{hex, primitives::Address};
use serde::{Deserialize, Serialize};

pub const FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Erc20TokenRow {
    pub chain_id: u32,
    pub orderbook_address: Address,
    pub token_address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

const ADDRESSES_CLAUSE: &str = "/*ADDRESSES_CLAUSE*/";
const ADDRESSES_CLAUSE_BODY: &str = "AND token_address IN ({list})";

/// Builds the SQL statement used to load ERC20 metadata for the supplied
/// addresses. Returns `Ok(None)` when the address list is empty to allow
/// callers to short-circuit database work.
pub fn build_fetch_stmt(
    ob_id: &OrderbookIdentifier,
    addresses: &[Address],
) -> Result<Option<SqlStatement>, SqlBuildError> {
    if addresses.is_empty() {
        return Ok(None);
    }

    let mut stmt = SqlStatement::new(FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL);
    // ?1: chain id
    stmt.push(ob_id.chain_id as i64);
    // ?2: orderbook address
    stmt.push(ob_id.orderbook_address.to_string());
    // IN list for addresses
    stmt.bind_list_clause(
        ADDRESSES_CLAUSE,
        ADDRESSES_CLAUSE_BODY,
        addresses
            .iter()
            .cloned()
            .map(|a| SqlValue::Text(hex::encode_prefixed(a))),
    )?;
    Ok(Some(stmt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::hex;
    #[test]
    fn empty_addresses_returns_none() {
        let q =
            build_fetch_stmt(&OrderbookIdentifier::new(1, Address::from([0xbe; 20])), &[]).unwrap();
        assert!(q.is_none());
    }

    #[test]
    fn builds_in_clause_and_chain_id_with_params() {
        let address_a = Address::from([0xab; 20]);
        let address_b = Address::from([0xcd; 20]);
        let addrs = vec![address_a, address_b];
        let orderbook = Address::from([0xbe; 20]);
        let stmt = build_fetch_stmt(&OrderbookIdentifier::new(137, orderbook), &addrs)
            .expect("should build")
            .unwrap();

        // SQL has markers resolved and ?1 present for chain id
        assert!(stmt.sql.contains("WHERE chain_id = ?1"));
        assert!(!stmt.sql.contains(ADDRESSES_CLAUSE));

        // Params: chain id, orderbook address, then addresses
        assert_eq!(stmt.params.len(), 4);
        assert_eq!(stmt.params[0], SqlValue::I64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(orderbook.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(hex::encode_prefixed(addrs[0]))
        );
        assert_eq!(
            stmt.params[3],
            SqlValue::Text(hex::encode_prefixed(addrs[1]))
        );
    }

    #[test]
    fn missing_addresses_marker_yields_error() {
        // Remove the clause marker from the template to simulate template drift.
        let bad_template = FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL.replace(ADDRESSES_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // ?1 is chain id
        stmt.push(1i64);
        let err = stmt
            .bind_list_clause(
                ADDRESSES_CLAUSE,
                ADDRESSES_CLAUSE_BODY,
                vec![SqlValue::Text("0xabc".into())],
            )
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }
}
