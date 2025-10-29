SELECT DISTINCT lower(store_address) AS store_address
FROM order_events
WHERE chain_id = ?1
  AND lower(orderbook_address) = lower(?2)
  AND store_address IS NOT NULL
  AND store_address != ''
UNION
SELECT DISTINCT lower(store_address) AS store_address
FROM interpreter_store_sets
WHERE chain_id = ?1
  AND lower(orderbook_address) = lower(?2)
  AND store_address IS NOT NULL
  AND store_address != '';
