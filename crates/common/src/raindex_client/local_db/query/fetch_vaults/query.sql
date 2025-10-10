SELECT
  o.chain_id,
  o.vault_id,
  o.token,
  o.owner,
  o.orderbook_address,
  et.name   AS token_name,
  et.symbol AS token_symbol,
  et.decimals AS token_decimals,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT ('0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = io.chain_id
       AND lower(oe.orderbook_address) = lower(io.orderbook_address)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.chain_id,
               e1.orderbook_address,
               e1.order_owner,
               e1.order_nonce,
               e1.event_type
        FROM order_events e1
        WHERE NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.chain_id = e1.chain_id
            AND lower(e2.orderbook_address) = lower(e1.orderbook_address)
            AND e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (
              e2.block_number > e1.block_number
              OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
            )
        )
      ) l
        ON l.chain_id = o.chain_id
       AND lower(l.orderbook_address) = lower(o.orderbook_address)
       AND l.order_owner = oe.order_owner
       AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = o.chain_id
        AND lower(io.orderbook_address) = lower(o.orderbook_address)
        AND io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'INPUT'
      ORDER BY oe.order_hash
    ) AS q_in
  ) AS input_orders,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT ('0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = io.chain_id
       AND lower(oe.orderbook_address) = lower(io.orderbook_address)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.chain_id,
               e1.orderbook_address,
               e1.order_owner,
               e1.order_nonce,
               e1.event_type
        FROM order_events e1
        WHERE NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.chain_id = e1.chain_id
            AND lower(e2.orderbook_address) = lower(e1.orderbook_address)
            AND e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (
              e2.block_number > e1.block_number
              OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
            )
        )
      ) l
        ON l.chain_id = o.chain_id
       AND lower(l.orderbook_address) = lower(o.orderbook_address)
       AND l.order_owner = oe.order_owner
       AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = o.chain_id
        AND lower(io.orderbook_address) = lower(o.orderbook_address)
        AND io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.order_hash
    ) AS q_out
  ) AS output_orders,
  COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.chain_id = o.chain_id
      AND lower(vd.orderbook_address) = lower(o.orderbook_address)
      AND vd.owner    = o.owner
      AND vd.token    = o.token
      AND vd.vault_id = o.vault_id
  ), FLOAT_ZERO_HEX()) AS balance
FROM (
  SELECT DISTINCT
    vd.chain_id,
    vd.orderbook_address,
    vd.owner,
    vd.token,
    vd.vault_id
  FROM vault_deltas vd
) AS o
JOIN erc20_tokens et
  ON et.chain_id = o.chain_id
 AND lower(et.address) = lower(o.token)
WHERE 1=1
  AND o.chain_id = ?chain_id
?filter_chain_ids
?filter_orderbooks
?filter_owners
?filter_tokens
?filter_hide_zero_balance
ORDER BY o.owner, o.token, o.vault_id;
