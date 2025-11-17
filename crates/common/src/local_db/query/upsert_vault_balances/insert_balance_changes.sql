INSERT OR IGNORE INTO vault_balance_changes (
  chain_id,
  orderbook_address,
  owner,
  token,
  vault_id,
  block_number,
  log_index,
  delta
)
SELECT
  vd.chain_id,
  vd.orderbook_address,
  vd.owner,
  vd.token,
  vd.vault_id,
  vd.block_number,
  vd.log_index,
  vd.delta
FROM vault_deltas vd
WHERE vd.chain_id = ?1
  AND vd.orderbook_address = ?2
  AND vd.block_number BETWEEN ?3 AND ?4;
