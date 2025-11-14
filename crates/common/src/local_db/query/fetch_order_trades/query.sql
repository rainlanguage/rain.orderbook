WITH
params AS (
  SELECT
    ?1 AS chain_id,
    lower(?2) AS orderbook_address,
    lower(?3) AS order_hash
),
order_add_events AS (
  SELECT
    oe.chain_id,
    oe.orderbook_address,
    lower(oe.orderbook_address) AS orderbook_address_lower,
    oe.transaction_hash,
    oe.log_index,
    oe.block_number,
    oe.block_timestamp,
    oe.order_owner,
    lower(oe.order_owner) AS order_owner_lower,
    oe.order_nonce,
    oe.order_hash,
    lower(oe.order_hash) AS order_hash_lower
  FROM order_events oe
  JOIN params p
    ON oe.chain_id = p.chain_id
   AND lower(oe.orderbook_address) = p.orderbook_address
   AND lower(oe.order_hash) = p.order_hash
  WHERE oe.event_type = 'AddOrderV3'
),
take_trades AS (
  SELECT
    'take' AS trade_kind,
    t.chain_id,
    t.orderbook_address,
    oe.order_hash,
    t.order_owner,
    t.order_nonce,
    t.transaction_hash,
    t.log_index,
    t.block_number,
    t.block_timestamp,
    t.sender AS transaction_sender,
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    t.taker_output AS input_delta,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(t.taker_input) AS output_delta
  FROM take_orders t
  JOIN params p
    ON t.chain_id = p.chain_id
   AND lower(t.orderbook_address) = p.orderbook_address
  JOIN order_add_events oe
    ON oe.chain_id = t.chain_id
   AND lower(oe.orderbook_address) = lower(t.orderbook_address)
   AND lower(oe.order_owner) = lower(t.order_owner)
   AND oe.order_nonce = t.order_nonce
   AND (
        oe.block_number < t.block_number
     OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
       AND lower(newer.orderbook_address) = lower(oe.orderbook_address)
       AND lower(newer.order_owner) = lower(oe.order_owner)
       AND newer.order_nonce = oe.order_nonce
       AND (
            newer.block_number < t.block_number
         OR (newer.block_number = t.block_number AND newer.log_index <= t.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND lower(io_in.orderbook_address) = lower(oe.orderbook_address)
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = t.input_io_index
   AND lower(io_in.io_type) = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND lower(io_out.orderbook_address) = lower(oe.orderbook_address)
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = t.output_io_index
   AND lower(io_out.io_type) = 'output'
),
clear_alice AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    c.chain_id,
    c.orderbook_address,
    oe.order_hash,
    oe.order_owner,
    oe.order_nonce,
    c.transaction_hash,
    c.log_index,
    c.block_number,
    c.block_timestamp,
    c.sender AS transaction_sender,
    c.alice_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.alice_input AS input_delta,
    c.alice_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.alice_output) AS output_delta
  FROM clear_v3_events c
  JOIN params p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN order_add_events oe
    ON oe.chain_id = c.chain_id
   AND lower(oe.orderbook_address) = lower(c.orderbook_address)
   AND lower(oe.order_hash) = lower(c.alice_order_hash)
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
       AND lower(newer.orderbook_address) = lower(oe.orderbook_address)
       AND lower(newer.order_hash) = lower(oe.order_hash)
       AND (
            newer.block_number < c.block_number
         OR (newer.block_number = c.block_number AND newer.log_index <= c.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
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
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND lower(io_in.orderbook_address) = lower(oe.orderbook_address)
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = c.alice_input_io_index
   AND lower(io_in.io_type) = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND lower(io_out.orderbook_address) = lower(oe.orderbook_address)
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = c.alice_output_io_index
   AND lower(io_out.io_type) = 'output'
  WHERE lower(c.alice_order_hash) = p.order_hash
),
clear_bob AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    c.chain_id,
    c.orderbook_address,
    oe.order_hash,
    oe.order_owner,
    oe.order_nonce,
    c.transaction_hash,
    c.log_index,
    c.block_number,
    c.block_timestamp,
    c.sender AS transaction_sender,
    c.bob_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.bob_input AS input_delta,
    c.bob_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.bob_output) AS output_delta
  FROM clear_v3_events c
  JOIN params p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN order_add_events oe
    ON oe.chain_id = c.chain_id
   AND lower(oe.orderbook_address) = lower(c.orderbook_address)
   AND lower(oe.order_hash) = lower(c.bob_order_hash)
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
       AND lower(newer.orderbook_address) = lower(oe.orderbook_address)
       AND lower(newer.order_hash) = lower(oe.order_hash)
       AND (
            newer.block_number < c.block_number
         OR (newer.block_number = c.block_number AND newer.log_index <= c.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
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
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND lower(io_in.orderbook_address) = lower(oe.orderbook_address)
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = c.bob_input_io_index
   AND lower(io_in.io_type) = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND lower(io_out.orderbook_address) = lower(oe.orderbook_address)
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = c.bob_output_io_index
   AND lower(io_out.io_type) = 'output'
  WHERE lower(c.bob_order_hash) = p.order_hash
),
clear_trades AS (
  SELECT * FROM clear_alice
  UNION ALL
  SELECT * FROM clear_bob
),
unioned_trades AS (
  SELECT * FROM take_trades
  UNION ALL
  SELECT * FROM clear_trades
),
trade_rows AS (
  SELECT
    ut.trade_kind,
    ut.chain_id,
    ut.orderbook_address,
    ut.order_hash,
    ut.order_owner,
    ut.order_nonce,
    ut.transaction_hash,
    ut.log_index,
    ut.block_number,
    ut.block_timestamp,
    ut.transaction_sender,
    ut.input_vault_id,
    ut.input_token,
    ut.input_delta,
    ut.output_vault_id,
    ut.output_token,
    ut.output_delta,
    lower(ut.orderbook_address) AS orderbook_address_lower,
    lower(ut.order_owner) AS order_owner_lower,
    lower(ut.input_token) AS input_token_lower,
    lower(ut.output_token) AS output_token_lower,
    lower(ut.input_vault_id) AS input_vault_lower,
    lower(ut.output_vault_id) AS output_vault_lower
  FROM unioned_trades ut
),
trade_with_snapshots AS (
  SELECT
    tr.*,
    mvb_in.balance AS input_base_balance,
    mvb_in.last_block AS input_base_block,
    mvb_in.last_log_index AS input_base_log_index,
    mvb_out.balance AS output_base_balance,
    mvb_out.last_block AS output_base_block,
    mvb_out.last_log_index AS output_base_log_index
  FROM trade_rows tr
  LEFT JOIN materialized_vault_balances mvb_in
    ON mvb_in.chain_id = tr.chain_id
   AND lower(mvb_in.orderbook_address) = tr.orderbook_address_lower
   AND lower(mvb_in.owner) = tr.order_owner_lower
   AND lower(mvb_in.token) = tr.input_token_lower
   AND lower(mvb_in.vault_id) = tr.input_vault_lower
  LEFT JOIN materialized_vault_balances mvb_out
    ON mvb_out.chain_id = tr.chain_id
   AND lower(mvb_out.orderbook_address) = tr.orderbook_address_lower
   AND lower(mvb_out.owner) = tr.order_owner_lower
   AND lower(mvb_out.token) = tr.output_token_lower
   AND lower(mvb_out.vault_id) = tr.output_vault_lower
)
SELECT
  tws.trade_kind,
  tws.orderbook_address,
  tws.order_hash,
  tws.order_owner,
  tws.order_nonce,
  tws.transaction_hash,
  tws.log_index,
  tws.block_number,
  tws.block_timestamp,
  tws.transaction_sender,
  tws.input_vault_id,
  tws.input_token,
  tok_in.name AS input_token_name,
  tok_in.symbol AS input_token_symbol,
  tok_in.decimals AS input_token_decimals,
  tws.input_delta,
  (
    SELECT COALESCE(FLOAT_SUM(prev.delta ORDER BY prev.block_number, prev.log_index), FLOAT_ZERO_HEX())
    FROM (
      SELECT
        tws.input_base_balance AS delta,
        tws.input_base_block AS block_number,
        tws.input_base_log_index AS log_index
      WHERE tws.input_base_block IS NOT NULL
      UNION ALL
      SELECT
        vd.delta,
        vd.block_number,
        vd.log_index
      FROM vault_deltas vd
      WHERE vd.chain_id = tws.chain_id
        AND lower(vd.orderbook_address) = tws.orderbook_address_lower
        AND lower(vd.owner) = tws.order_owner_lower
        AND lower(vd.token) = tws.input_token_lower
        AND lower(vd.vault_id) = tws.input_vault_lower
        AND (
             tws.input_base_block IS NULL
          OR vd.block_number > tws.input_base_block
          OR (vd.block_number = tws.input_base_block AND vd.log_index > tws.input_base_log_index)
        )
        AND (
             vd.block_number < tws.block_number
          OR (vd.block_number = tws.block_number AND vd.log_index <= tws.log_index)
        )
      ORDER BY block_number, log_index
    ) AS prev
  ) AS input_running_balance,
  tws.output_vault_id,
  tws.output_token,
  tok_out.name AS output_token_name,
  tok_out.symbol AS output_token_symbol,
  tok_out.decimals AS output_token_decimals,
  tws.output_delta,
  (
    SELECT COALESCE(FLOAT_SUM(prev.delta ORDER BY prev.block_number, prev.log_index), FLOAT_ZERO_HEX())
    FROM (
      SELECT
        tws.output_base_balance AS delta,
        tws.output_base_block AS block_number,
        tws.output_base_log_index AS log_index
      WHERE tws.output_base_block IS NOT NULL
      UNION ALL
      SELECT
        vd.delta,
        vd.block_number,
        vd.log_index
      FROM vault_deltas vd
      WHERE vd.chain_id = tws.chain_id
        AND lower(vd.orderbook_address) = tws.orderbook_address_lower
        AND lower(vd.owner) = tws.order_owner_lower
        AND lower(vd.token) = tws.output_token_lower
        AND lower(vd.vault_id) = tws.output_vault_lower
        AND (
             tws.output_base_block IS NULL
          OR vd.block_number > tws.output_base_block
          OR (vd.block_number = tws.output_base_block AND vd.log_index > tws.output_base_log_index)
        )
        AND (
             vd.block_number < tws.block_number
          OR (vd.block_number = tws.block_number AND vd.log_index <= tws.log_index)
        )
      ORDER BY block_number, log_index
    ) AS prev
  ) AS output_running_balance,
  lower(
    '0x' ||
    CASE
      WHEN lower(substr(tws.transaction_hash, 1, 2)) = '0x' THEN substr(tws.transaction_hash, 3)
      ELSE tws.transaction_hash
    END ||
    printf('%016x', tws.log_index)
  ) AS trade_id
FROM trade_with_snapshots tws
LEFT JOIN erc20_tokens tok_in
  ON tok_in.chain_id = tws.chain_id
 AND lower(tok_in.orderbook_address) = tws.orderbook_address_lower
 AND lower(tok_in.token_address) = tws.input_token_lower
LEFT JOIN erc20_tokens tok_out
  ON tok_out.chain_id = tws.chain_id
 AND lower(tok_out.orderbook_address) = tws.orderbook_address_lower
 AND lower(tok_out.token_address) = tws.output_token_lower
WHERE 1 = 1
/*START_TS_CLAUSE*/
/*END_TS_CLAUSE*/
ORDER BY tws.block_timestamp DESC, tws.block_number DESC, tws.log_index DESC, tws.trade_kind;
