CREATE VIEW IF NOT EXISTS vault_deltas AS
WITH add_order_events AS (
  SELECT *
  FROM order_events
  WHERE event_type = 'AddOrderV3'
),
latest_order_events AS (
  SELECT oe.*
  FROM add_order_events oe
  WHERE NOT EXISTS (
    SELECT 1
    FROM add_order_events newer
    WHERE newer.chain_id = oe.chain_id
      AND lower(newer.orderbook_address) = lower(oe.orderbook_address)
      AND newer.order_hash = oe.order_hash
      AND (
           newer.block_number > oe.block_number
        OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
      )
  )
)
SELECT d.chain_id,
       d.orderbook_address,
       d.transaction_hash,
       d.log_index,
       d.block_number,
       d.block_timestamp,
       d.sender AS owner,
       'DEPOSIT' AS kind,
       d.token,
       d.vault_id,
       d.deposit_amount AS delta
FROM deposits d

UNION ALL
SELECT w.chain_id,
       w.orderbook_address,
       w.transaction_hash,
       w.log_index,
       w.block_number,
       w.block_timestamp,
       w.sender,
       'WITHDRAW',
       w.token,
       w.vault_id,
       FLOAT_NEGATE(w.withdraw_amount)
FROM withdrawals w

UNION ALL
-- maker buy: +taker_output
SELECT t.chain_id,
       t.orderbook_address,
       t.transaction_hash,
       t.log_index,
       t.block_number,
       t.block_timestamp,
       t.order_owner,
       'TAKE_INPUT',
       io.token,
       io.vault_id,
       t.taker_output
FROM take_orders t
JOIN add_order_events oe
  ON oe.chain_id = t.chain_id
 AND lower(oe.orderbook_address) = lower(t.orderbook_address)
 AND oe.order_owner = t.order_owner
 AND oe.order_nonce = t.order_nonce
 AND (oe.block_number < t.block_number
   OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index))
 AND NOT EXISTS (
   SELECT 1 FROM add_order_events x
   WHERE x.chain_id = t.chain_id
     AND lower(x.orderbook_address) = lower(t.orderbook_address)
     AND x.order_owner = t.order_owner
     AND x.order_nonce = t.order_nonce
     AND (x.block_number < t.block_number
       OR (x.block_number = t.block_number AND x.log_index <= t.log_index))
     AND (x.block_number > oe.block_number
       OR (x.block_number = oe.block_number AND x.log_index > oe.log_index))
 )
JOIN order_ios io
  ON io.chain_id = oe.chain_id
 AND lower(io.orderbook_address) = lower(oe.orderbook_address)
 AND io.transaction_hash = oe.transaction_hash
 AND io.log_index = oe.log_index
 AND io.io_index = t.input_io_index
 AND UPPER(io.io_type) = 'INPUT'

UNION ALL
-- maker sell: -taker_input
SELECT t.chain_id,
       t.orderbook_address,
       t.transaction_hash,
       t.log_index,
       t.block_number,
       t.block_timestamp,
       t.order_owner,
       'TAKE_OUTPUT',
       io.token,
       io.vault_id,
       FLOAT_NEGATE(t.taker_input)
FROM take_orders t
JOIN add_order_events oe
  ON oe.chain_id = t.chain_id
 AND lower(oe.orderbook_address) = lower(t.orderbook_address)
 AND oe.order_owner = t.order_owner
 AND oe.order_nonce = t.order_nonce
 AND (oe.block_number < t.block_number
   OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index))
 AND NOT EXISTS (
   SELECT 1 FROM add_order_events x
   WHERE x.chain_id = t.chain_id
     AND lower(x.orderbook_address) = lower(t.orderbook_address)
     AND x.order_owner = t.order_owner
     AND x.order_nonce = t.order_nonce
     AND (x.block_number < t.block_number
       OR (x.block_number = t.block_number AND x.log_index <= t.log_index))
     AND (x.block_number > oe.block_number
       OR (x.block_number = oe.block_number AND x.log_index > oe.log_index))
 )
JOIN order_ios io
  ON io.chain_id = oe.chain_id
 AND lower(io.orderbook_address) = lower(oe.orderbook_address)
 AND io.transaction_hash = oe.transaction_hash
 AND io.log_index = oe.log_index
 AND io.io_index = t.output_io_index
 AND UPPER(io.io_type)='OUTPUT'

UNION ALL
-- clears (maker-oriented already)
SELECT DISTINCT c.chain_id,
       c.orderbook_address,
       c.transaction_hash,
       c.log_index,
       c.block_number,
       c.block_timestamp,
       c.alice_order_owner,
       'CLEAR_ALICE_INPUT',
       io_ai.token,
       c.alice_input_vault_id,
       a.alice_input
FROM clear_v3_events c
JOIN after_clear_v2_events a
  ON a.chain_id = c.chain_id
 AND lower(a.orderbook_address) = lower(c.orderbook_address)
 AND a.transaction_hash = c.transaction_hash
 AND a.log_index = (
     SELECT MIN(ac.log_index)
     FROM after_clear_v2_events ac
     WHERE ac.chain_id = c.chain_id
       AND lower(ac.orderbook_address) = lower(c.orderbook_address)
       AND ac.transaction_hash = c.transaction_hash
       AND ac.log_index > c.log_index
 )
JOIN latest_order_events oeA
  ON oeA.chain_id = c.chain_id
 AND lower(oeA.orderbook_address) = lower(c.orderbook_address)
 AND oeA.order_hash = c.alice_order_hash
JOIN order_ios io_ai
  ON io_ai.chain_id = oeA.chain_id
 AND lower(io_ai.orderbook_address) = lower(oeA.orderbook_address)
 AND io_ai.transaction_hash = oeA.transaction_hash
 AND io_ai.log_index = oeA.log_index
 AND io_ai.io_index = c.alice_input_io_index
 AND UPPER(io_ai.io_type)='INPUT'

UNION ALL
SELECT DISTINCT c.chain_id,
       c.orderbook_address,
       c.transaction_hash,
       c.log_index,
       c.block_number,
       c.block_timestamp,
       c.alice_order_owner,
       'CLEAR_ALICE_OUTPUT',
       io_ao.token,
       c.alice_output_vault_id,
       FLOAT_NEGATE(a.alice_output)
FROM clear_v3_events c
JOIN after_clear_v2_events a
  ON a.chain_id = c.chain_id
 AND lower(a.orderbook_address) = lower(c.orderbook_address)
 AND a.transaction_hash = c.transaction_hash
 AND a.log_index = (
     SELECT MIN(ac.log_index)
     FROM after_clear_v2_events ac
     WHERE ac.chain_id = c.chain_id
       AND lower(ac.orderbook_address) = lower(c.orderbook_address)
       AND ac.transaction_hash = c.transaction_hash
       AND ac.log_index > c.log_index
 )
JOIN latest_order_events oeA
  ON oeA.chain_id = c.chain_id
 AND lower(oeA.orderbook_address) = lower(c.orderbook_address)
 AND oeA.order_hash = c.alice_order_hash
JOIN order_ios io_ao
  ON io_ao.chain_id = oeA.chain_id
 AND lower(io_ao.orderbook_address) = lower(oeA.orderbook_address)
 AND io_ao.transaction_hash = oeA.transaction_hash
 AND io_ao.log_index = oeA.log_index
 AND io_ao.io_index = c.alice_output_io_index
 AND UPPER(io_ao.io_type)='OUTPUT'

UNION ALL
SELECT DISTINCT c.chain_id,
       c.orderbook_address,
       c.transaction_hash,
       c.log_index,
       c.block_number,
       c.block_timestamp,
       c.bob_order_owner,
       'CLEAR_BOB_INPUT',
       io_bi.token,
       c.bob_input_vault_id,
       a.bob_input
FROM clear_v3_events c
JOIN after_clear_v2_events a
  ON a.chain_id = c.chain_id
 AND lower(a.orderbook_address) = lower(c.orderbook_address)
 AND a.transaction_hash = c.transaction_hash
 AND a.log_index = (
     SELECT MIN(ac.log_index)
     FROM after_clear_v2_events ac
     WHERE ac.chain_id = c.chain_id
       AND lower(ac.orderbook_address) = lower(c.orderbook_address)
       AND ac.transaction_hash = c.transaction_hash
       AND ac.log_index > c.log_index
 )
JOIN latest_order_events oeB
  ON oeB.chain_id = c.chain_id
 AND lower(oeB.orderbook_address) = lower(c.orderbook_address)
 AND oeB.order_hash = c.bob_order_hash
JOIN order_ios io_bi
  ON io_bi.chain_id = oeB.chain_id
 AND lower(io_bi.orderbook_address) = lower(oeB.orderbook_address)
 AND io_bi.transaction_hash = oeB.transaction_hash
 AND io_bi.log_index = oeB.log_index
 AND io_bi.io_index = c.bob_input_io_index
 AND UPPER(io_bi.io_type)='INPUT'

UNION ALL
SELECT DISTINCT c.chain_id,
       c.orderbook_address,
       c.transaction_hash,
       c.log_index,
       c.block_number,
       c.block_timestamp,
       c.bob_order_owner,
       'CLEAR_BOB_OUTPUT',
       io_bo.token,
       c.bob_output_vault_id,
       FLOAT_NEGATE(a.bob_output)
FROM clear_v3_events c
JOIN after_clear_v2_events a
  ON a.chain_id = c.chain_id
 AND lower(a.orderbook_address) = lower(c.orderbook_address)
 AND a.transaction_hash = c.transaction_hash
 AND a.log_index = (
     SELECT MIN(ac.log_index)
     FROM after_clear_v2_events ac
     WHERE ac.chain_id = c.chain_id
       AND lower(ac.orderbook_address) = lower(c.orderbook_address)
       AND ac.transaction_hash = c.transaction_hash
       AND ac.log_index > c.log_index
 )
JOIN latest_order_events oeB
  ON oeB.chain_id = c.chain_id
 AND lower(oeB.orderbook_address) = lower(c.orderbook_address)
 AND oeB.order_hash = c.bob_order_hash
JOIN order_ios io_bo
  ON io_bo.chain_id = oeB.chain_id
 AND lower(io_bo.orderbook_address) = lower(oeB.orderbook_address)
 AND io_bo.transaction_hash = oeB.transaction_hash
 AND io_bo.log_index = oeB.log_index
 AND io_bo.io_index = c.bob_output_io_index
 AND UPPER(io_bo.io_type)='OUTPUT'

UNION ALL
SELECT *
FROM (
  SELECT DISTINCT
    c.chain_id,
    c.orderbook_address,
    c.transaction_hash,
    c.log_index,
    c.block_number,
    c.block_timestamp,
    c.sender AS owner,
    'CLEAR_ALICE_BOUNTY' AS kind,
    io_ao.token,
    c.alice_bounty_vault_id AS vault_id,
    (
      SELECT COALESCE(FLOAT_SUM(val), FLOAT_ZERO_HEX())
      FROM (
        SELECT a.alice_output AS val
        UNION ALL
        SELECT FLOAT_NEGATE(a.bob_input)
      )
    ) AS delta
  FROM clear_v3_events c
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND lower(a.orderbook_address) = lower(c.orderbook_address)
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = c.chain_id
         AND lower(ac.orderbook_address) = lower(c.orderbook_address)
         AND ac.transaction_hash = c.transaction_hash
         AND ac.log_index > c.log_index
  )
  JOIN latest_order_events oeA
    ON oeA.chain_id = c.chain_id
   AND lower(oeA.orderbook_address) = lower(c.orderbook_address)
   AND oeA.order_hash = c.alice_order_hash
  JOIN order_ios io_ao
    ON io_ao.chain_id = oeA.chain_id
   AND lower(io_ao.orderbook_address) = lower(oeA.orderbook_address)
   AND io_ao.transaction_hash = oeA.transaction_hash
   AND io_ao.log_index = oeA.log_index
   AND io_ao.io_index = c.alice_output_io_index
   AND UPPER(io_ao.io_type)='OUTPUT'
) AS alice_bounty
WHERE NOT FLOAT_IS_ZERO(alice_bounty.delta)

UNION ALL
SELECT *
FROM (
  SELECT DISTINCT
    c.chain_id,
    c.orderbook_address,
    c.transaction_hash,
    c.log_index,
    c.block_number,
    c.block_timestamp,
    c.sender AS owner,
    'CLEAR_BOB_BOUNTY' AS kind,
    io_bo.token,
    c.bob_bounty_vault_id AS vault_id,
    (
      SELECT COALESCE(FLOAT_SUM(val), FLOAT_ZERO_HEX())
      FROM (
        SELECT a.bob_output AS val
        UNION ALL
        SELECT FLOAT_NEGATE(a.alice_input)
      )
    ) AS delta
  FROM clear_v3_events c
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND lower(a.orderbook_address) = lower(c.orderbook_address)
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = c.chain_id
         AND lower(ac.orderbook_address) = lower(c.orderbook_address)
         AND ac.transaction_hash = c.transaction_hash
         AND ac.log_index > c.log_index
  )
  JOIN latest_order_events oeB
    ON oeB.chain_id = c.chain_id
   AND lower(oeB.orderbook_address) = lower(c.orderbook_address)
   AND oeB.order_hash = c.bob_order_hash
  JOIN order_ios io_bo
    ON io_bo.chain_id = oeB.chain_id
   AND lower(io_bo.orderbook_address) = lower(oeB.orderbook_address)
   AND io_bo.transaction_hash = oeB.transaction_hash
   AND io_bo.log_index = oeB.log_index
   AND io_bo.io_index = c.bob_output_io_index
   AND UPPER(io_bo.io_type)='OUTPUT'
) AS bob_bounty
WHERE NOT FLOAT_IS_ZERO(bob_bounty.delta);
