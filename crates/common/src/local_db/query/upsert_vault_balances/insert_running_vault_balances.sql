WITH delta_batches AS (
  SELECT
    vd.chain_id,
    vd.orderbook_address,
    vd.owner,
    vd.token,
    vd.vault_id,
    COALESCE(
      FLOAT_SUM(vd.delta ORDER BY vd.block_number, vd.log_index),
      FLOAT_ZERO_HEX()
    ) AS balance_delta,
    MAX(vd.block_number) AS last_block,
    MAX(vd.log_index) AS last_log_index
  FROM vault_deltas vd
  WHERE vd.chain_id = ?1
    AND vd.orderbook_address = ?2
    AND vd.block_number BETWEEN ?3 AND ?4
  GROUP BY vd.chain_id, vd.orderbook_address, vd.owner, vd.token, vd.vault_id
),
existing_matching AS (
  SELECT
    mvb.chain_id,
    mvb.orderbook_address,
    mvb.owner,
    mvb.token,
    mvb.vault_id,
    mvb.balance AS balance_value,
    mvb.last_block,
    mvb.last_log_index
  FROM running_vault_balances mvb
  JOIN delta_batches db
    ON db.chain_id = mvb.chain_id
   AND db.orderbook_address = mvb.orderbook_address
   AND db.owner = mvb.owner
   AND db.token = mvb.token
   AND db.vault_id = mvb.vault_id
),
combined AS (
  SELECT
    chain_id,
    orderbook_address,
    owner,
    token,
    vault_id,
    balance_delta AS contribution,
    last_block,
    last_log_index
  FROM delta_batches
  UNION ALL
  SELECT
    chain_id,
    orderbook_address,
    owner,
    token,
    vault_id,
    balance_value AS contribution,
    last_block,
    last_log_index
  FROM existing_matching
),
aggregated AS (
  SELECT
    chain_id,
    orderbook_address,
    owner,
    token,
    vault_id,
    COALESCE(FLOAT_SUM(contribution), FLOAT_ZERO_HEX()) AS balance,
    MAX(last_block) AS last_block
  FROM combined
  GROUP BY chain_id, orderbook_address, owner, token, vault_id
)
INSERT OR REPLACE INTO running_vault_balances (
  chain_id,
  orderbook_address,
  owner,
  token,
  vault_id,
  balance,
  last_block,
  last_log_index,
  updated_at
)
SELECT
  a.chain_id,
  a.orderbook_address,
  a.owner,
  a.token,
  a.vault_id,
  a.balance,
  a.last_block,
  (
    SELECT c.last_log_index
    FROM combined c
    WHERE c.chain_id = a.chain_id
      AND c.orderbook_address = a.orderbook_address
      AND c.owner = a.owner
      AND c.token = a.token
      AND c.vault_id = a.vault_id
      AND c.last_block = a.last_block
    ORDER BY c.last_log_index DESC
    LIMIT 1
  ) AS last_log_index,
  (CAST(strftime('%s', 'now') AS INTEGER) * 1000) AS updated_at
FROM aggregated a;
