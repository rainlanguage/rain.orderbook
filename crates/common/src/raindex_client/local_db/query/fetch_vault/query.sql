SELECT
  os.chain_id,
  os.vault_id,
  os.token,
  os.owner,
  '?orderbook_address' AS orderbook_address,
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
        ON l.chain_id = os.chain_id
       AND lower(l.orderbook_address) = os.orderbook_address
       AND l.order_owner = oe.order_owner
       AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = os.chain_id
        AND lower(io.orderbook_address) = os.orderbook_address
        AND io.token    = os.token
        AND io.vault_id = os.vault_id
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
        ON l.chain_id = os.chain_id
       AND lower(l.orderbook_address) = os.orderbook_address
       AND l.order_owner = oe.order_owner
       AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = os.chain_id
        AND lower(io.orderbook_address) = os.orderbook_address
        AND io.token    = os.token
        AND io.vault_id = os.vault_id
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.order_hash
    ) AS q_out
  ) AS output_orders,
  COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.chain_id = os.chain_id
      AND lower(vd.orderbook_address) = os.orderbook_address
      AND vd.owner    = os.owner
      AND vd.token    = os.token
      AND vd.vault_id = os.vault_id
  ), FLOAT_ZERO_HEX()) AS balance
FROM (
  SELECT
    ?chain_id AS chain_id,
    lower('?orderbook_address') AS orderbook_address,
    '?vault_id' AS vault_id,
    '?token' AS token,
    COALESCE(
      (
        SELECT oe.order_owner
        FROM order_ios io
        JOIN order_events oe
          ON oe.chain_id = io.chain_id
         AND lower(oe.orderbook_address) = lower(io.orderbook_address)
         AND oe.transaction_hash = io.transaction_hash
         AND oe.log_index = io.log_index
        WHERE io.chain_id = ?chain_id
          AND lower(io.orderbook_address) = lower('?orderbook_address')
          AND io.token = '?token'
          AND io.vault_id = '?vault_id'
        ORDER BY oe.block_number DESC, oe.log_index DESC
        LIMIT 1
      ),
      (
        SELECT owner FROM (
          SELECT d.sender AS owner, d.block_number, d.log_index
          FROM deposits d
          WHERE d.chain_id = ?chain_id
            AND lower(d.orderbook_address) = lower('?orderbook_address')
            AND d.token = '?token'
            AND d.vault_id = '?vault_id'
          UNION ALL
          SELECT w.sender AS owner, w.block_number, w.log_index
          FROM withdrawals w
          WHERE w.chain_id = ?chain_id
            AND lower(w.orderbook_address) = lower('?orderbook_address')
            AND w.token = '?token'
            AND w.vault_id = '?vault_id'
          ORDER BY block_number DESC, log_index DESC
          LIMIT 1
        )
      )
    ) AS owner
) AS os
JOIN erc20_tokens et
  ON et.chain_id = os.chain_id
 AND lower(et.address) = lower(os.token);
