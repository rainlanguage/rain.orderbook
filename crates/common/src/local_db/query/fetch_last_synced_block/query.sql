SELECT
    chain_id,
    orderbook_address,
    last_synced_block,
    updated_at
FROM sync_status
WHERE chain_id = ?1
  AND orderbook_address = ?2;
