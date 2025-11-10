SELECT
    chain_id,
    orderbook_address,
    last_synced_block,
    updated_at
FROM sync_status
WHERE chain_id = ?1
  AND lower(orderbook_address) = lower(?2);
