INSERT INTO target_watermarks (chain_id, orderbook_address, last_block, last_hash)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(chain_id, orderbook_address) DO UPDATE SET
  last_block = excluded.last_block,
  last_hash = excluded.last_hash,
  updated_at = CURRENT_TIMESTAMP;
