SELECT DISTINCT lower(store_address) AS store_address
FROM order_events
WHERE store_address IS NOT NULL
  AND store_address != ''
  AND chain_id = ?chain_id
  AND lower(orderbook_address) = lower('?orderbook_address')
UNION
SELECT DISTINCT lower(store_address) AS store_address
FROM interpreter_store_sets
WHERE store_address IS NOT NULL
  AND store_address != ''
  AND chain_id = ?chain_id
  AND lower(orderbook_address) = lower('?orderbook_address');
