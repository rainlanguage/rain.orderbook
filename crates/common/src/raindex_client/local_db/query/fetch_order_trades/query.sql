SELECT
  trade_kind,
  chain_id,
  orderbook_address,
  order_hash,
  order_owner,
  order_nonce,
  transaction_hash,
  log_index,
  block_number,
  block_timestamp,
  transaction_sender,
  input_vault_id,
  input_token,
  input_token_name,
  input_token_symbol,
  input_token_decimals,
  input_delta,
  input_running_balance,
  output_vault_id,
  output_token,
  output_token_name,
  output_token_symbol,
  output_token_decimals,
  output_delta,
  output_running_balance,
  '0x' || lower(replace(transaction_hash, '0x', '')) || printf('%016x', log_index) AS trade_id
FROM (
  SELECT
    'take' AS trade_kind,
    t.chain_id,
    t.orderbook_address,
    oe.order_hash,
    oe.order_owner,
    oe.order_nonce,
    t.transaction_hash,
    t.log_index,
    t.block_number,
    t.block_timestamp,
    t.sender AS transaction_sender,
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    et_in.name AS input_token_name,
    et_in.symbol AS input_token_symbol,
    et_in.decimals AS input_token_decimals,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    et_out.name AS output_token_name,
    et_out.symbol AS output_token_symbol,
    et_out.decimals AS output_token_decimals,
    COALESCE(vd_in.delta, t.taker_output) AS input_delta,
    CASE
      WHEN vd_in.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_in.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_in.orderbook_address)
          AND lower(vd2.owner) = lower(vd_in.owner)
          AND lower(vd2.token) = lower(vd_in.token)
          AND vd2.vault_id = vd_in.vault_id
          AND (
            vd2.block_number < vd_in.block_number
            OR (vd2.block_number = vd_in.block_number AND vd2.log_index <= vd_in.log_index)
          )
      )
      ELSE NULL
    END AS input_running_balance,
    COALESCE(vd_out.delta, FLOAT_NEGATE(t.taker_input)) AS output_delta,
    CASE
      WHEN vd_out.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_out.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_out.orderbook_address)
          AND lower(vd2.owner) = lower(vd_out.owner)
          AND lower(vd2.token) = lower(vd_out.token)
          AND vd2.vault_id = vd_out.vault_id
          AND (
            vd2.block_number < vd_out.block_number
            OR (vd2.block_number = vd_out.block_number AND vd2.log_index <= vd_out.log_index)
          )
      )
      ELSE NULL
    END AS output_running_balance
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
  LEFT JOIN vault_deltas vd_in
    ON vd_in.chain_id = t.chain_id
   AND lower(vd_in.orderbook_address) = lower(t.orderbook_address)
   AND lower(vd_in.transaction_hash) = lower(t.transaction_hash)
   AND vd_in.log_index = t.log_index
   AND lower(vd_in.owner) = lower(oe.order_owner)
   AND lower(vd_in.token) = lower(io_in.token)
   AND vd_in.vault_id = io_in.vault_id
   AND vd_in.kind = 'TAKE_INPUT'
  LEFT JOIN vault_deltas vd_out
    ON vd_out.chain_id = t.chain_id
   AND lower(vd_out.orderbook_address) = lower(t.orderbook_address)
   AND lower(vd_out.transaction_hash) = lower(t.transaction_hash)
   AND vd_out.log_index = t.log_index
   AND lower(vd_out.owner) = lower(oe.order_owner)
   AND lower(vd_out.token) = lower(io_out.token)
   AND vd_out.vault_id = io_out.vault_id
   AND vd_out.kind = 'TAKE_OUTPUT'
  LEFT JOIN erc20_tokens et_in
    ON et_in.chain_id = t.chain_id
   AND lower(et_in.address) = lower(io_in.token)
  LEFT JOIN erc20_tokens et_out
    ON et_out.chain_id = t.chain_id
   AND lower(et_out.address) = lower(io_out.token)

  UNION ALL

  SELECT
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
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    et_in.name AS input_token_name,
    et_in.symbol AS input_token_symbol,
    et_in.decimals AS input_token_decimals,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    et_out.name AS output_token_name,
    et_out.symbol AS output_token_symbol,
    et_out.decimals AS output_token_decimals,
    COALESCE(vd_in.delta, a.alice_input) AS input_delta,
    CASE
      WHEN vd_in.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_in.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_in.orderbook_address)
          AND lower(vd2.owner) = lower(vd_in.owner)
          AND lower(vd2.token) = lower(vd_in.token)
          AND vd2.vault_id = vd_in.vault_id
          AND (
            vd2.block_number < vd_in.block_number
            OR (vd2.block_number = vd_in.block_number AND vd2.log_index <= vd_in.log_index)
          )
      )
      ELSE NULL
    END AS input_running_balance,
    COALESCE(vd_out.delta, FLOAT_NEGATE(a.alice_output)) AS output_delta,
    CASE
      WHEN vd_out.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_out.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_out.orderbook_address)
          AND lower(vd2.owner) = lower(vd_out.owner)
          AND lower(vd2.token) = lower(vd_out.token)
          AND vd2.vault_id = vd_out.vault_id
          AND (
            vd2.block_number < vd_out.block_number
            OR (vd2.block_number = vd_out.block_number AND vd2.log_index <= vd_out.log_index)
          )
      )
      ELSE NULL
    END AS output_running_balance
  FROM clear_v3_events c
  JOIN (
    SELECT
      ?chain_id AS chain_id,
      lower('?orderbook_address') AS orderbook_address,
      lower('?order_hash') AS order_hash
  ) AS p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND lower(a.orderbook_address) = lower(c.orderbook_address)
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = c.log_index
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
  LEFT JOIN vault_deltas vd_in
    ON vd_in.chain_id = c.chain_id
   AND lower(vd_in.orderbook_address) = lower(c.orderbook_address)
   AND lower(vd_in.transaction_hash) = lower(c.transaction_hash)
   AND vd_in.log_index = c.log_index
   AND lower(vd_in.owner) = lower(c.alice_order_owner)
   AND lower(vd_in.token) = lower(io_in.token)
   AND vd_in.vault_id = io_in.vault_id
   AND vd_in.kind = 'CLEAR_ALICE_INPUT'
  LEFT JOIN vault_deltas vd_out
    ON vd_out.chain_id = c.chain_id
   AND lower(vd_out.orderbook_address) = lower(c.orderbook_address)
   AND lower(vd_out.transaction_hash) = lower(c.transaction_hash)
   AND vd_out.log_index = c.log_index
   AND lower(vd_out.owner) = lower(c.alice_order_owner)
   AND lower(vd_out.token) = lower(io_out.token)
   AND vd_out.vault_id = io_out.vault_id
   AND vd_out.kind = 'CLEAR_ALICE_OUTPUT'
  LEFT JOIN erc20_tokens et_in
    ON et_in.chain_id = c.chain_id
   AND lower(et_in.address) = lower(io_in.token)
  LEFT JOIN erc20_tokens et_out
    ON et_out.chain_id = c.chain_id
   AND lower(et_out.address) = lower(io_out.token)
  WHERE lower(c.alice_order_hash) = p.order_hash

  UNION ALL

  SELECT
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
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    et_in.name AS input_token_name,
    et_in.symbol AS input_token_symbol,
    et_in.decimals AS input_token_decimals,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    et_out.name AS output_token_name,
    et_out.symbol AS output_token_symbol,
    et_out.decimals AS output_token_decimals,
    COALESCE(vd_in.delta, a.bob_input) AS input_delta,
    CASE
      WHEN vd_in.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_in.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_in.orderbook_address)
          AND lower(vd2.owner) = lower(vd_in.owner)
          AND lower(vd2.token) = lower(vd_in.token)
          AND vd2.vault_id = vd_in.vault_id
          AND (
            vd2.block_number < vd_in.block_number
            OR (vd2.block_number = vd_in.block_number AND vd2.log_index <= vd_in.log_index)
          )
      )
      ELSE NULL
    END AS input_running_balance,
    COALESCE(vd_out.delta, FLOAT_NEGATE(a.bob_output)) AS output_delta,
    CASE
      WHEN vd_out.transaction_hash IS NOT NULL THEN (
        SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
        FROM vault_deltas vd2
        WHERE vd2.chain_id = vd_out.chain_id
          AND lower(vd2.orderbook_address) = lower(vd_out.orderbook_address)
          AND lower(vd2.owner) = lower(vd_out.owner)
          AND lower(vd2.token) = lower(vd_out.token)
          AND vd2.vault_id = vd_out.vault_id
          AND (
            vd2.block_number < vd_out.block_number
            OR (vd2.block_number = vd_out.block_number AND vd2.log_index <= vd_out.log_index)
          )
      )
      ELSE NULL
    END AS output_running_balance
  FROM clear_v3_events c
  JOIN (
    SELECT
      ?chain_id AS chain_id,
      lower('?orderbook_address') AS orderbook_address,
      lower('?order_hash') AS order_hash
  ) AS p
    ON c.chain_id = p.chain_id
   AND lower(c.orderbook_address) = p.orderbook_address
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND lower(a.orderbook_address) = lower(c.orderbook_address)
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = c.log_index
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
  LEFT JOIN vault_deltas vd_in
    ON vd_in.chain_id = c.chain_id
   AND lower(vd_in.orderbook_address) = lower(c.orderbook_address)
   AND lower(vd_in.transaction_hash) = lower(c.transaction_hash)
   AND vd_in.log_index = c.log_index
   AND lower(vd_in.owner) = lower(c.bob_order_owner)
   AND lower(vd_in.token) = lower(io_in.token)
   AND vd_in.vault_id = io_in.vault_id
   AND vd_in.kind = 'CLEAR_BOB_INPUT'
  LEFT JOIN vault_deltas vd_out
    ON vd_out.chain_id = c.chain_id
   AND lower(vd_out.orderbook_address) = lower(c.orderbook_address)
   AND lower(vd_out.transaction_hash) = lower(c.transaction_hash)
   AND vd_out.log_index = c.log_index
   AND lower(vd_out.owner) = lower(c.bob_order_owner)
   AND lower(vd_out.token) = lower(io_out.token)
   AND vd_out.vault_id = io_out.vault_id
   AND vd_out.kind = 'CLEAR_BOB_OUTPUT'
  LEFT JOIN erc20_tokens et_in
    ON et_in.chain_id = c.chain_id
   AND lower(et_in.address) = lower(io_in.token)
  LEFT JOIN erc20_tokens et_out
    ON et_out.chain_id = c.chain_id
   AND lower(et_out.address) = lower(io_out.token)
  WHERE lower(c.bob_order_hash) = p.order_hash
) AS combined_trades
WHERE 1=1
?filter_start_timestamp
?filter_end_timestamp
ORDER BY block_timestamp DESC, block_number DESC, log_index DESC;
