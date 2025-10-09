UPDATE sync_status
SET last_synced_block = ?block_number,
    updated_at = CURRENT_TIMESTAMP
WHERE chain_id = ?chain_id
  AND orderbook_address = '?orderbook_address';
