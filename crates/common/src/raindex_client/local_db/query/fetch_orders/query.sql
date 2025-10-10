SELECT
    COALESCE(la.chain_id, l.chain_id) AS chain_id,
    COALESCE(la.orderbook_address, l.orderbook_address) AS orderbook_address,
    COALESCE(la.order_hash, l.order_hash) AS order_hash,
    l.order_owner AS owner,
    fa.block_timestamp AS block_timestamp,
    fa.block_number AS block_number,
    COALESCE(la.order_bytes, l.order_bytes) AS order_bytes,
    GROUP_CONCAT(
        CASE
            WHEN ios.io_type = 'input' THEN ios.io_index || ':' || ios.vault_id || ':' || ios.token
        END
    ) AS inputs,
    GROUP_CONCAT(
        CASE
            WHEN ios.io_type = 'output' THEN ios.io_index || ':' || ios.vault_id || ':' || ios.token
        END
    ) AS outputs,
    (
        SELECT COUNT(*) FROM take_orders t
        WHERE t.chain_id = COALESCE(la.chain_id, l.chain_id)
          AND lower(t.orderbook_address) = lower(COALESCE(la.orderbook_address, l.orderbook_address))
          AND t.order_owner = l.order_owner
          AND t.order_nonce = l.order_nonce
    )
    + (
        SELECT COUNT(*) FROM clear_v3_events c
        WHERE c.chain_id = COALESCE(la.chain_id, l.chain_id)
          AND lower(c.orderbook_address) = lower(COALESCE(la.orderbook_address, l.orderbook_address))
          AND (
                lower(c.alice_order_hash) = lower(COALESCE(la.order_hash, l.order_hash))
             OR lower(c.bob_order_hash) = lower(COALESCE(la.order_hash, l.order_hash))
          )
    ) AS trade_count,
    (l.event_type = 'AddOrderV3') AS active,
    COALESCE(la.transaction_hash, l.transaction_hash) AS transaction_hash,
    (
        SELECT m.meta
        FROM meta_events m
        WHERE m.chain_id = COALESCE(la.chain_id, l.chain_id)
          AND lower(m.orderbook_address) = lower(COALESCE(la.orderbook_address, l.orderbook_address))
          AND lower(m.subject) = lower(COALESCE(la.order_hash, l.order_hash))
        ORDER BY m.block_number DESC, m.log_index DESC
        LIMIT 1
    ) AS meta
FROM order_events l
LEFT JOIN (
    SELECT e1.chain_id,
           e1.orderbook_address,
           e1.order_owner,
           e1.order_nonce,
           e1.transaction_hash,
           e1.log_index,
           e1.order_hash,
           e1.order_bytes
    FROM order_events e1
    WHERE e1.event_type = 'AddOrderV3'
      AND NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.event_type = 'AddOrderV3'
            AND e2.chain_id = e1.chain_id
            AND e2.orderbook_address = e1.orderbook_address
            AND e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (
                e2.block_number > e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
            )
      )
) la
  ON la.chain_id = l.chain_id
 AND la.orderbook_address = l.orderbook_address
 AND la.order_owner = l.order_owner
 AND la.order_nonce = l.order_nonce
LEFT JOIN order_ios ios
  ON ios.chain_id = la.chain_id
 AND ios.orderbook_address = la.orderbook_address
 AND ios.transaction_hash = la.transaction_hash
 AND ios.log_index = la.log_index
LEFT JOIN (
    SELECT e1.chain_id,
           e1.orderbook_address,
           e1.order_owner,
           e1.order_nonce,
           e1.block_timestamp,
           e1.block_number
    FROM order_events e1
    WHERE e1.event_type = 'AddOrderV3'
      AND NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.event_type = 'AddOrderV3'
            AND e2.chain_id = e1.chain_id
            AND e2.orderbook_address = e1.orderbook_address
            AND e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (
                e2.block_number < e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index < e1.log_index)
            )
      )
) fa
  ON fa.chain_id = l.chain_id
 AND fa.orderbook_address = l.orderbook_address
 AND fa.order_owner = l.order_owner
 AND fa.order_nonce = l.order_nonce
WHERE
NOT EXISTS (
    SELECT 1 FROM order_events e2
    WHERE e2.chain_id = l.chain_id
      AND e2.orderbook_address = l.orderbook_address
      AND e2.order_owner = l.order_owner
      AND e2.order_nonce = l.order_nonce
      AND (
          e2.block_number > l.block_number
          OR (e2.block_number = l.block_number AND e2.log_index > l.log_index)
      )
)
AND (
    '?filter_active' = 'all'
    OR ('?filter_active' = 'active' AND l.event_type = 'AddOrderV3')
    OR ('?filter_active' = 'inactive' AND l.event_type = 'RemoveOrderV3')
)
?filter_chain_ids
?filter_orderbooks
?filter_owners
?filter_order_hash
?filter_tokens
GROUP BY
    COALESCE(la.chain_id, l.chain_id),
    COALESCE(la.orderbook_address, l.orderbook_address),
    COALESCE(la.order_hash, l.order_hash),
    l.order_owner,
    fa.block_timestamp,
    fa.block_number,
    l.order_nonce,
    l.event_type,
    COALESCE(la.order_bytes, l.order_bytes),
    COALESCE(la.transaction_hash, l.transaction_hash)
ORDER BY fa.block_timestamp DESC;
