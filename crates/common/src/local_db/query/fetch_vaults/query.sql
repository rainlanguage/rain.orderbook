SELECT
  o.vault_id,
  o.token,
  o.owner,
  ?2 AS orderbook_address,
  et.name   AS token_name,
  et.symbol AS token_symbol,
  et.decimals AS token_decimals,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT (
        '0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = ?1
       AND lower(oe.orderbook_address) = lower(?2)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE e1.chain_id = ?1
          AND lower(e1.orderbook_address) = lower(?2)
          AND NOT EXISTS (
            SELECT 1 FROM order_events e2
            WHERE e2.chain_id = ?1
              AND lower(e2.orderbook_address) = lower(?2)
              AND e2.order_owner = e1.order_owner
              AND e2.order_nonce = e1.order_nonce
              AND (
                e2.block_number > e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
              )
          )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = ?1
        AND lower(io.orderbook_address) = lower(?2)
        AND io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'INPUT'
      ORDER BY oe.order_hash
    ) AS q_in
  ) AS input_orders,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT (
        '0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = ?1
       AND lower(oe.orderbook_address) = lower(?2)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE e1.chain_id = ?1
          AND lower(e1.orderbook_address) = lower(?2)
          AND NOT EXISTS (
            SELECT 1 FROM order_events e2
            WHERE e2.chain_id = ?1
              AND lower(e2.orderbook_address) = lower(?2)
              AND e2.order_owner = e1.order_owner
              AND e2.order_nonce = e1.order_nonce
              AND (
                e2.block_number > e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
              )
          )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = ?1
        AND lower(io.orderbook_address) = lower(?2)
        AND io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.order_hash
    ) AS q_out
  ) AS output_orders,
  COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.chain_id = ?1
      AND lower(vd.orderbook_address) = lower(?2)
      AND vd.owner    = o.owner
      AND vd.token    = o.token
      AND vd.vault_id = o.vault_id
  ), FLOAT_ZERO_HEX()) AS balance
FROM (
  /* all distinct (owner, token, vault_id) that ever had a delta on this target */
  SELECT DISTINCT owner, token, vault_id
  FROM vault_deltas
  WHERE chain_id = ?1
    AND lower(orderbook_address) = lower(?2)
) AS o
JOIN erc20_tokens et
  ON et.chain_id = ?1
 AND lower(et.orderbook_address) = lower(?2)
 AND lower(et.token_address) = lower(o.token)
WHERE 1=1
/*OWNERS_CLAUSE*/
/*TOKENS_CLAUSE*/
/*HIDE_ZERO_BALANCE*/
ORDER BY o.owner, o.token, o.vault_id;
