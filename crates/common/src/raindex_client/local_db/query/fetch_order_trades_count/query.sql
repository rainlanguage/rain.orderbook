SELECT COUNT(*) AS trade_count
FROM (
  SELECT t.transaction_hash, t.log_index, t.block_timestamp
  FROM take_orders t
  JOIN (
    SELECT
      ?chain_id AS chain_id,
      lower('?orderbook_address') AS orderbook_address,
      lower('?order_hash') AS order_hash
  ) AS p
    ON t.chain_id = p.chain_id
   AND lower(t.orderbook_address) = p.orderbook_address
  JOIN order_events oe
    ON oe.chain_id = t.chain_id
   AND lower(oe.orderbook_address) = lower(t.orderbook_address)
   AND lower(oe.order_hash) = p.order_hash
   AND oe.order_owner = t.order_owner
   AND oe.order_nonce = t.order_nonce
   AND (
        oe.block_number < t.block_number
     OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = oe.chain_id
       AND lower(oe2.orderbook_address) = lower(oe.orderbook_address)
       AND lower(oe2.order_hash) = p.order_hash
       AND oe2.order_owner = t.order_owner
       AND oe2.order_nonce = t.order_nonce
       AND (
            oe2.block_number < t.block_number
         OR (oe2.block_number = t.block_number AND oe2.log_index <= t.log_index)
       )
       AND (
            oe2.block_number > oe.block_number
         OR (oe2.block_number = oe.block_number AND oe2.log_index > oe.log_index)
       )
   )

  UNION ALL

  SELECT c.transaction_hash, c.log_index, c.block_timestamp
  FROM clear_v3_events c
  JOIN (
    SELECT
      ?chain_id AS chain_id,
      lower('?orderbook_address') AS orderbook_address,
      lower('?order_hash') AS order_hash
  ) AS p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN order_events oe
    ON oe.chain_id = c.chain_id
   AND lower(oe.orderbook_address) = lower(c.orderbook_address)
   AND lower(oe.order_hash) = p.order_hash
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = oe.chain_id
       AND lower(oe2.orderbook_address) = lower(oe.orderbook_address)
       AND lower(oe2.order_hash) = p.order_hash
       AND (
            oe2.block_number < c.block_number
         OR (oe2.block_number = c.block_number AND oe2.log_index <= c.log_index)
       )
       AND (
            oe2.block_number > oe.block_number
         OR (oe2.block_number = oe.block_number AND oe2.log_index > oe.log_index)
       )
   )
  WHERE lower(c.alice_order_hash) = p.order_hash

  UNION ALL

  SELECT c.transaction_hash, c.log_index, c.block_timestamp
  FROM clear_v3_events c
  JOIN (
    SELECT
      ?chain_id AS chain_id,
      lower('?orderbook_address') AS orderbook_address,
      lower('?order_hash') AS order_hash
  ) AS p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN order_events oe
    ON oe.chain_id = c.chain_id
   AND lower(oe.orderbook_address) = lower(c.orderbook_address)
   AND lower(oe.order_hash) = p.order_hash
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = oe.chain_id
       AND lower(oe2.orderbook_address) = lower(oe.orderbook_address)
       AND lower(oe2.order_hash) = p.order_hash
       AND (
            oe2.block_number < c.block_number
         OR (oe2.block_number = c.block_number AND oe2.log_index <= c.log_index)
       )
       AND (
            oe2.block_number > oe.block_number
         OR (oe2.block_number = oe.block_number AND oe2.log_index > oe.log_index)
       )
   )
  WHERE lower(c.bob_order_hash) = p.order_hash
) AS combined_trades
WHERE 1=1
?filter_start_timestamp
?filter_end_timestamp;
