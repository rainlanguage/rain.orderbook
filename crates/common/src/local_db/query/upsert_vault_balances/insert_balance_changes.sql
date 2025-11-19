INSERT OR IGNORE INTO vault_balance_changes (
  chain_id,
  orderbook_address,
  transaction_hash,
  owner,
  token,
  vault_id,
  block_number,
  block_timestamp,
  log_index,
  change_type,
  delta
)
SELECT
  vd.chain_id,
  vd.orderbook_address,
  vd.transaction_hash,
  vd.owner,
  vd.token,
  vd.vault_id,
  vd.block_number,
  vd.block_timestamp,
  vd.log_index,
  vd.kind AS change_type,
  vd.delta
FROM vault_deltas vd
WHERE vd.chain_id = ?1
  AND vd.orderbook_address = ?2
  AND vd.block_number BETWEEN ?3 AND ?4;
