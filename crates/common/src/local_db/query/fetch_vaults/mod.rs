const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub owners: Vec<String>,
    pub tokens: Vec<String>,
    pub hide_zero_balance: bool,
}

pub fn build_fetch_vaults_query(chain_id: u32, args: &FetchVaultsArgs) -> String {
    let sanitize_literal = |value: &str| value.replace('\'', "''");

    let owner_values: Vec<String> = args
        .owners
        .iter()
        .filter_map(|owner| {
            let trimmed = owner.trim();
            if trimmed.is_empty() {
                None
            } else {
                let lowered = trimmed.to_lowercase();
                Some(format!("'{}'", sanitize_literal(&lowered)))
            }
        })
        .collect();
    let filter_owners = if owner_values.is_empty() {
        String::new()
    } else {
        format!("\nAND lower(o.owner) IN ({})\n", owner_values.join(", "))
    };

    let token_values: Vec<String> = args
        .tokens
        .iter()
        .filter_map(|token| {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                None
            } else {
                let lowered = trimmed.to_lowercase();
                Some(format!("'{}'", sanitize_literal(&lowered)))
            }
        })
        .collect();
    let filter_tokens = if token_values.is_empty() {
        String::new()
    } else {
        format!("\nAND lower(o.token) IN ({})\n", token_values.join(", "))
    };

    const BALANCE_EXPR: &str = r"#COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.owner    = o.owner
      AND vd.token    = o.token
      AND vd.vault_id = o.vault_id
  ), FLOAT_ZERO_HEX())
        #";

    let filter_hide_zero_balance = if args.hide_zero_balance {
        format!("\nAND NOT FLOAT_IS_ZERO({expr})\n", expr = BALANCE_EXPR)
    } else {
        String::new()
    };

    QUERY_TEMPLATE
        .replace("?chain_id", &chain_id.to_string())
        .replace("?filter_owners", &filter_owners)
        .replace("?filter_tokens", &filter_tokens)
        .replace("?filter_hide_zero_balance", &filter_hide_zero_balance)
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
        let q = build_fetch_vaults_query(1, &args);
        // In this query the placeholder appears inside quotes, so result should be '1'
        assert!(q.contains("et.chain_id = '1'"));
        assert!(!q.contains("?filter_owners"));
        assert!(!q.contains("?filter_tokens"));
        assert!(!q.contains("?filter_hide_zero_balance"));
    }

    #[test]
    fn owners_tokens_and_hide_zero() {
        let mut args = mk_args();
        args.owners = vec![" 0xA ".into(), "O'Owner".into()];
        args.tokens = vec!["TOK'A".into()];
        args.hide_zero_balance = true;
        let q = build_fetch_vaults_query(137, &args);

        // Owners IN clause
        assert!(q.contains("AND lower(o.owner) IN ('0xa', 'o''owner')"));
        // Tokens IN clause
        assert!(q.contains("AND lower(o.token) IN ('tok''a')"));
        // Hide-zero filter inserted
        assert!(q.contains("AND NOT FLOAT_IS_ZERO("));

        // Placeholders removed
        assert!(!q.contains("?filter_owners"));
        assert!(!q.contains("?filter_tokens"));
        assert!(!q.contains("?filter_hide_zero_balance"));
    }
}
