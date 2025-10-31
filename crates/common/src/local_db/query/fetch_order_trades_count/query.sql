SELECT COUNT(*) AS trade_count
FROM (
  SELECT t.transaction_hash, t.log_index, t.block_timestamp
  FROM take_orders t
  JOIN order_events oe
    ON oe.chain_id = ?1
   AND lower(oe.orderbook_address) = lower(?2)
   AND lower(oe.order_hash) = lower(?3)
   AND oe.order_owner = t.order_owner
   AND oe.order_nonce = t.order_nonce
   AND (
        oe.block_number < t.block_number
     OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = ?1
       AND lower(oe2.orderbook_address) = lower(?2)
       AND lower(oe2.order_hash) = lower(?3)
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
  WHERE t.chain_id = ?1
    AND lower(t.orderbook_address) = lower(?2)

  UNION ALL

  SELECT c.transaction_hash, c.log_index, c.block_timestamp
  FROM clear_v3_events c
  JOIN order_events oe
    ON oe.chain_id = ?1
   AND lower(oe.orderbook_address) = lower(?2)
   AND lower(oe.order_hash) = lower(c.alice_order_hash)
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = ?1
       AND lower(oe2.orderbook_address) = lower(?2)
       AND lower(oe2.order_hash) = lower(c.alice_order_hash)
       AND (
            oe2.block_number < c.block_number
         OR (oe2.block_number = c.block_number AND oe2.log_index <= c.log_index)
       )
       AND (
            oe2.block_number > oe.block_number
         OR (oe2.block_number = oe.block_number AND oe2.log_index > oe.log_index)
       )
   )
  WHERE c.chain_id = ?1
    AND lower(c.orderbook_address) = lower(?2)
    AND lower(c.alice_order_hash) = lower(?3)

  UNION ALL

  SELECT c.transaction_hash, c.log_index, c.block_timestamp
  FROM clear_v3_events c
  JOIN order_events oe
    ON oe.chain_id = ?1
   AND lower(oe.orderbook_address) = lower(?2)
   AND lower(oe.order_hash) = lower(c.bob_order_hash)
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events oe2
     WHERE oe2.chain_id = ?1
       AND lower(oe2.orderbook_address) = lower(?2)
       AND lower(oe2.order_hash) = lower(c.bob_order_hash)
       AND (
            oe2.block_number < c.block_number
         OR (oe2.block_number = c.block_number AND oe2.log_index <= c.log_index)
       )
       AND (
            oe2.block_number > oe.block_number
         OR (oe2.block_number = oe.block_number AND oe2.log_index > oe.log_index)
       )
   )
  WHERE c.chain_id = ?1
    AND lower(c.orderbook_address) = lower(?2)
    AND lower(c.bob_order_hash) = lower(?3)
) AS combined_trades
WHERE 1=1
/*START_TS_CLAUSE*/
/*END_TS_CLAUSE*/;
