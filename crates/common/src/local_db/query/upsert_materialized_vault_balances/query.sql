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
  HAVING NOT FLOAT_IS_ZERO(balance_delta)
)
INSERT INTO materialized_vault_balances (
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
  chain_id,
  orderbook_address,
  owner,
  token,
  vault_id,
  balance_delta,
  last_block,
  last_log_index,
  (CAST(strftime('%s', 'now') AS INTEGER) * 1000) AS updated_at
FROM delta_batches
ON CONFLICT (chain_id, orderbook_address, owner, token, vault_id)
DO UPDATE SET
  balance = FLOAT_ADD(materialized_vault_balances.balance, excluded.balance),
  last_block = CASE
    WHEN excluded.last_block > materialized_vault_balances.last_block
      OR (
        excluded.last_block = materialized_vault_balances.last_block
        AND excluded.last_log_index > materialized_vault_balances.last_log_index
      )
    THEN excluded.last_block
    ELSE materialized_vault_balances.last_block
  END,
  last_log_index = CASE
    WHEN excluded.last_block > materialized_vault_balances.last_block
      OR (
        excluded.last_block = materialized_vault_balances.last_block
        AND excluded.last_log_index > materialized_vault_balances.last_log_index
      )
    THEN excluded.last_log_index
    ELSE materialized_vault_balances.last_log_index
  END,
  updated_at = excluded.updated_at;
