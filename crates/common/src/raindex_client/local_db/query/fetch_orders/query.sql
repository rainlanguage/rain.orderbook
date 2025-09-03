SELECT
COALESCE(la.order_hash, l.order_hash) AS order_hash,
l.order_owner AS owner,
fa.creation_time AS creation_time,
fa.creation_block AS creation_block,
GROUP_CONCAT(CASE WHEN ios.io_type = 'input' THEN ios.vault_id || ':' || ios.token END) AS inputs,
GROUP_CONCAT(CASE WHEN ios.io_type = 'output' THEN ios.vault_id || ':' || ios.token END) AS outputs,
(
    SELECT COUNT(*) FROM take_orders t
    WHERE t.order_owner = l.order_owner AND t.order_nonce = l.order_nonce
) AS trade_count,
CASE WHEN l.event_type = 'AddOrderV3' THEN 'active' ELSE 'inactive' END AS status
FROM order_events l
LEFT JOIN (
SELECT e1.order_owner, e1.order_nonce, e1.transaction_hash, e1.log_index, e1.order_hash
FROM order_events e1
WHERE e1.event_type = 'AddOrderV3'
    AND NOT EXISTS (
      SELECT 1 FROM order_events e2
      WHERE e2.event_type = 'AddOrderV3'
        AND e2.order_owner = e1.order_owner
        AND e2.order_nonce = e1.order_nonce
        AND (e2.block_number > e1.block_number
          OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index))
    )
) la ON la.order_owner = l.order_owner AND la.order_nonce = l.order_nonce
LEFT JOIN order_ios ios
ON ios.transaction_hash = la.transaction_hash AND ios.log_index = la.log_index
LEFT JOIN (
SELECT e1.order_owner, e1.order_nonce,
         e1.block_timestamp AS creation_time,
         e1.block_number AS creation_block
FROM order_events e1
WHERE e1.event_type = 'AddOrderV3'
    AND NOT EXISTS (
      SELECT 1 FROM order_events e2
      WHERE e2.event_type = 'AddOrderV3'
        AND e2.order_owner = e1.order_owner
        AND e2.order_nonce = e1.order_nonce
        AND (e2.block_number < e1.block_number
          OR (e2.block_number = e1.block_number AND e2.log_index < e1.log_index))
    )
) fa ON fa.order_owner = l.order_owner AND fa.order_nonce = l.order_nonce
WHERE
NOT EXISTS (
    SELECT 1 FROM order_events e2
    WHERE e2.order_owner = l.order_owner
      AND e2.order_nonce = l.order_nonce
      AND (e2.block_number > l.block_number
        OR (e2.block_number = l.block_number AND e2.log_index > l.log_index))
)
AND (
    '?filter' = 'all'
    OR ('?filter' = 'active' AND l.event_type = 'AddOrderV3')
    OR ('?filter' = 'inactive' AND l.event_type = 'RemoveOrderV3')
)
GROUP BY
COALESCE(la.order_hash, l.order_hash),
l.order_owner,
fa.creation_time,
fa.creation_block,
l.order_nonce,
l.event_type
ORDER BY fa.creation_time DESC;
