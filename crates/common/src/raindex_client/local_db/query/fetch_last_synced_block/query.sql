SELECT chain_id, orderbook_address, last_synced_block, updated_at
FROM sync_status
WHERE chain_id = ?chain_id
  AND orderbook_address = '?orderbook_address'
ORDER BY updated_at DESC
LIMIT 1;
