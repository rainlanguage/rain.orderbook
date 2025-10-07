SELECT
  block_timestamp AS block_timestamp,
  transaction_hash AS transaction_hash,
  deposit_amount_uint256 AS balance_change,
  'deposit' AS balance_change_type,
  block_number,
  log_index
FROM deposits
WHERE lower(vault_id) = lower(?vault_id)
  AND lower(token) = lower(?token)

UNION ALL

SELECT
  block_timestamp AS block_timestamp,
  transaction_hash AS transaction_hash,
  '-' || withdraw_amount_uint256 AS balance_change,
  'withdrawal' AS balance_change_type,
  block_number,
  log_index
FROM withdrawals
WHERE lower(vault_id) = lower(?vault_id)
  AND lower(token) = lower(?token)

UNION ALL

SELECT
  t.block_timestamp AS block_timestamp,
  t.transaction_hash AS transaction_hash,
  t.output AS balance_change,
  'take_order' AS balance_change_type,
  t.block_number,
  t.log_index
FROM take_orders t
JOIN order_events oe
  ON t.order_owner = oe.order_owner
 AND t.order_nonce = oe.order_nonce
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE oi.vault_id = '0x11ed774b75522e6056c27c575431b224f8ef03b99f2ab177b26df5c35c1c287f'
  AND lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_index = t.input_io_index
  AND oi.io_type  = 'input'

UNION ALL

SELECT
  t.block_timestamp AS block_timestamp,
  t.transaction_hash AS transaction_hash,
  '-' || t.input AS balance_change,
  'take_order' AS balance_change_type,
  t.block_number,
  t.log_index
FROM take_orders t
JOIN order_events oe
  ON t.order_owner = oe.order_owner
 AND t.order_nonce = oe.order_nonce
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_index = t.output_io_index
  AND oi.io_type  = 'output'

UNION ALL

SELECT
  c.block_timestamp AS block_timestamp,
  c.transaction_hash AS transaction_hash,
  '-' || ac.alice_output AS balance_change,
  'clear' AS balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v3_events c
JOIN after_clear_v2_events ac
  ON c.transaction_hash = ac.transaction_hash
 AND c.sender          = ac.sender
JOIN order_events oe
  ON c.alice_order_hash = oe.order_hash
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE oi.vault_id = c.alice_output_vault_id
  AND lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_type = 'output'
  AND oi.io_index = c.alice_output_io_index

UNION ALL

SELECT
  c.block_timestamp AS block_timestamp,
  c.transaction_hash AS transaction_hash,
  ac.alice_input AS balance_change,
  'clear' AS balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v3_events c
JOIN after_clear_v2_events ac
  ON c.transaction_hash = ac.transaction_hash
 AND c.sender          = ac.sender
JOIN order_events oe
  ON c.alice_order_hash = oe.order_hash
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE oi.vault_id = c.alice_input_vault_id
  AND lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_type = 'input'
  AND oi.io_index = c.alice_input_io_index

UNION ALL

SELECT
  c.block_timestamp AS block_timestamp,
  c.transaction_hash AS transaction_hash,
  '-' || ac.bob_output AS balance_change,
  'clear' AS balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v3_events c
JOIN after_clear_v2_events ac
  ON c.transaction_hash = ac.transaction_hash
 AND c.sender          = ac.sender
JOIN order_events oe
  ON c.bob_order_hash = oe.order_hash
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE oi.vault_id = c.bob_output_vault_id
  AND lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_type = 'output'
  AND oi.io_index = c.bob_output_io_index

UNION ALL

SELECT
  c.block_timestamp AS block_timestamp,
  c.transaction_hash AS transaction_hash,
  ac.bob_input AS balance_change,
  'clear' AS balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v3_events c
JOIN after_clear_v2_events ac
  ON c.transaction_hash = ac.transaction_hash
 AND c.sender          = ac.sender
JOIN order_events oe
  ON c.bob_order_hash = oe.order_hash
JOIN order_ios oi
  ON oe.transaction_hash = oi.transaction_hash
 AND oe.log_index = oi.log_index
WHERE oi.vault_id = c.bob_input_vault_id
  AND lower(oi.vault_id) = lower(?vault_id)
  AND lower(oi.token)   = lower(?token)
  AND oi.io_type = 'input'
  AND oi.io_index = c.bob_input_io_index

ORDER BY block_number DESC, log_index DESC;
