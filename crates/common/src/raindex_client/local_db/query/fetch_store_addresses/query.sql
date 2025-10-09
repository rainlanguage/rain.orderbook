SELECT DISTINCT lower(store_address) AS store_address
FROM order_events
WHERE store_address IS NOT NULL AND store_address != ''
UNION
SELECT DISTINCT lower(store_address) AS store_address
FROM interpreter_store_sets
WHERE store_address IS NOT NULL AND store_address != '';
