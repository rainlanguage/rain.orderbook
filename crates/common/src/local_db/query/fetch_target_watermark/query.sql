SELECT chain_id, orderbook_address, last_block, last_hash, updated_at
FROM target_watermarks
WHERE chain_id = ?1 AND lower(orderbook_address) = lower(?2);
