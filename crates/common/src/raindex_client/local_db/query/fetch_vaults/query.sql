SELECT
  o.vault_id,
  o.token,
  o.owner,
  '0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB' AS orderbook_address,
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
        ON oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (e2.block_number > e1.block_number
              OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index))
        )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.token    = o.token
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
        ON oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE NOT EXISTS (
          SELECT 1 FROM order_events e2
          WHERE e2.order_owner = e1.order_owner
            AND e2.order_nonce = e1.order_nonce
            AND (e2.block_number > e1.block_number
              OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index))
        )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.order_hash
    ) AS q_out
  ) AS output_orders,
  COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.owner    = o.owner
      AND vd.token    = o.token
      AND vd.vault_id = o.vault_id
  ), FLOAT_ZERO()) AS balance
FROM (
  /* all distinct (owner, token, vault_id) that ever had a delta */
  SELECT DISTINCT owner, token, vault_id
  FROM vault_deltas
) AS o
JOIN erc20_tokens et
  ON et.chain_id = '?chain_id'
 AND lower(et.address) = lower(o.token)
WHERE 1=1
?filter_owners
?filter_tokens
?filter_hide_zero_balance
ORDER BY o.owner, o.token, o.vault_id;
