SELECT 
  datetime(CASE WHEN block_timestamp = 0 THEN NULL ELSE block_timestamp END, 'unixepoch') as date,
  transaction_hash as tx_hash,
  amount as balance_change,
  'deposit' as balance_change_type,
  block_number,
  log_index
FROM deposits
WHERE vault_id = ?vault_id AND token = ?token

UNION ALL

SELECT 
  datetime(CASE WHEN block_timestamp = 0 THEN NULL ELSE block_timestamp END, 'unixepoch') as date,
  transaction_hash as tx_hash,
  '-' || amount as balance_change,
  'withdrawal' as balance_change_type,
  block_number,
  log_index
FROM withdrawals
WHERE vault_id = ?vault_id AND token = ?token

UNION ALL

SELECT 
  datetime(CASE WHEN t.block_timestamp = 0 THEN NULL ELSE t.block_timestamp END, 'unixepoch') as date,
  t.transaction_hash as tx_hash,
  t.input_amount as balance_change,
  'take_order' as balance_change_type,
  t.block_number,
  t.log_index
FROM take_orders t
JOIN order_events oe ON t.order_owner = oe.owner AND t.order_nonce = oe.nonce  
JOIN order_ios oi ON oe.id = oi.order_event_id 
WHERE oi.vault_id = ?vault_id 
  AND oi.token = ?token
  AND oi.io_index = t.input_io_index 
  AND oi.io_type = 'input'

UNION ALL

SELECT 
  datetime(CASE WHEN t.block_timestamp = 0 THEN NULL ELSE t.block_timestamp END, 'unixepoch') as date,
  t.transaction_hash as tx_hash,
  '-' || t.output_amount as balance_change,
  'take_order' as balance_change_type,
  t.block_number,
  t.log_index
FROM take_orders t
JOIN order_events oe ON t.order_owner = oe.owner AND t.order_nonce = oe.nonce
JOIN order_ios oi ON oe.id = oi.order_event_id
WHERE oi.vault_id = ?vault_id 
  AND oi.token = ?token
  AND oi.io_index = t.output_io_index 
  AND oi.io_type = 'output'

UNION ALL

SELECT 
  datetime(CASE WHEN c.block_timestamp = 0 THEN NULL ELSE c.block_timestamp END, 'unixepoch') as date,
  c.transaction_hash as tx_hash,
  '-' || ac.alice_output as balance_change,
  'clear' as balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v2_events c
JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
JOIN order_events oe ON c.alice_order_hash = oe.order_hash
JOIN order_ios oi ON oe.id = oi.order_event_id 
WHERE oi.vault_id = c.alice_output_vault_id 
  AND oi.vault_id = ?vault_id
  AND oi.token = ?token
  AND oi.io_type = 'output'

UNION ALL

SELECT 
  datetime(CASE WHEN c.block_timestamp = 0 THEN NULL ELSE c.block_timestamp END, 'unixepoch') as date,
  c.transaction_hash as tx_hash,
  ac.alice_input as balance_change,
  'clear' as balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v2_events c
JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
JOIN order_events oe ON c.alice_order_hash = oe.order_hash
JOIN order_ios oi ON oe.id = oi.order_event_id 
WHERE oi.vault_id = c.alice_input_vault_id 
  AND oi.vault_id = ?vault_id
  AND oi.token = ?token
  AND oi.io_type = 'input'

UNION ALL

SELECT 
  datetime(CASE WHEN c.block_timestamp = 0 THEN NULL ELSE c.block_timestamp END, 'unixepoch') as date,
  c.transaction_hash as tx_hash,
  '-' || ac.bob_output as balance_change,
  'clear' as balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v2_events c
JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
JOIN order_events oe ON c.bob_order_hash = oe.order_hash
JOIN order_ios oi ON oe.id = oi.order_event_id 
WHERE oi.vault_id = c.bob_output_vault_id 
  AND oi.vault_id = ?vault_id
  AND oi.token = ?token
  AND oi.io_type = 'output'

UNION ALL

SELECT 
  datetime(CASE WHEN c.block_timestamp = 0 THEN NULL ELSE c.block_timestamp END, 'unixepoch') as date,
  c.transaction_hash as tx_hash,
  ac.bob_input as balance_change,
  'clear' as balance_change_type,
  c.block_number,
  c.log_index
FROM clear_v2_events c
JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
JOIN order_events oe ON c.bob_order_hash = oe.order_hash
JOIN order_ios oi ON oe.id = oi.order_event_id 
WHERE oi.vault_id = c.bob_input_vault_id 
  AND oi.vault_id = ?vault_id
  AND oi.token = ?token
  AND oi.io_type = 'input'

ORDER BY block_number DESC, log_index DESC;