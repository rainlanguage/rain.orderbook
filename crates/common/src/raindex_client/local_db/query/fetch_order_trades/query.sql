SELECT *
FROM (
  SELECT
    t.block_timestamp AS block_timestamp,
    t.block_number AS block_number,
    t.transaction_hash,
    t.output AS input_amount,
    t.input AS output_amount
  FROM order_events o
  JOIN take_orders t
    ON lower(o.order_owner) = lower(t.order_owner)
   AND o.order_nonce = t.order_nonce
  WHERE lower(o.order_hash) = lower(?order_hash)

  UNION ALL

  SELECT
    c.block_timestamp AS block_timestamp,
    c.block_number AS block_number,
    c.transaction_hash,
    CASE
      WHEN lower(c.alice_order_hash) = lower(?order_hash) THEN a.alice_input
      ELSE a.bob_input
    END AS input_amount,
    CASE
      WHEN lower(c.alice_order_hash) = lower(?order_hash) THEN a.alice_output
      ELSE a.bob_output
    END AS output_amount
  FROM clear_v3_events c
  JOIN after_clear_v2_events a
    ON a.transaction_hash = c.transaction_hash
   AND a.sender = c.sender
  WHERE lower(c.alice_order_hash) = lower(?order_hash)
     OR lower(c.bob_order_hash) = lower(?order_hash)
)
ORDER BY block_timestamp DESC;
