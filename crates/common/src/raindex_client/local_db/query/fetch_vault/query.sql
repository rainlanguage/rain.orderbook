SELECT
  d.vault_id,
  d.token,
  d.sender AS owner,
  '0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB' AS orderbook_address,
  COALESCE(b.balance, '0') AS balance,
  GROUP_CONCAT(DISTINCT CASE WHEN oi2.io_type = 'input' THEN oe2.order_hash END) AS input_order_hashes,
  GROUP_CONCAT(DISTINCT CASE WHEN oi2.io_type = 'output' THEN oe2.order_hash END) AS output_order_hashes
FROM (
  SELECT DISTINCT vault_id, token, sender
  FROM deposits
  WHERE lower(vault_id) = lower('?vault_id') AND lower(token) = lower('?token')
) d
LEFT JOIN (
  SELECT vault_id, token, FLOAT_SUM(amount_change) AS balance
  FROM (
      SELECT vault_id, token, deposit_amount_uint256 AS amount_change
      FROM deposits
      WHERE lower(vault_id) = lower('?vault_id') AND lower(token) = lower('?token')
      UNION ALL
      SELECT vault_id, token, '-' || withdraw_amount_uint256 AS amount_change
      FROM withdrawals
      WHERE lower(vault_id) = lower('?vault_id') AND lower(token) = lower('?token')
      UNION ALL
      SELECT oi.vault_id, oi.token, t.input AS amount_change
      FROM take_orders t
      JOIN order_events oe
        ON t.order_owner = oe.order_owner AND t.order_nonce = oe.order_nonce
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
      WHERE oi.io_index = t.input_io_index AND oi.io_type = 'input'
        AND lower(oi.vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
      UNION ALL
      SELECT oi.vault_id, oi.token, '-' || t.output AS amount_change
      FROM take_orders t
      JOIN order_events oe
        ON t.order_owner = oe.order_owner AND t.order_nonce = oe.order_nonce
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
      WHERE oi.io_index = t.output_io_index AND oi.io_type = 'output'
        AND lower(oi.vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
      UNION ALL
      SELECT c.alice_output_vault_id AS vault_id,
             oi.token,
             '-' || ac.alice_output AS amount_change
      FROM clear_v3_events c
      JOIN after_clear_v2_events ac
        ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
      JOIN order_events oe
        ON oe.order_hash = c.alice_order_hash AND oe.order_owner = c.alice_order_owner
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
       AND oi.vault_id = c.alice_output_vault_id AND oi.io_type = 'output'
      WHERE lower(c.alice_output_vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
      UNION ALL
      SELECT c.alice_input_vault_id AS vault_id,
             oi.token,
             ac.alice_input AS amount_change
      FROM clear_v3_events c
      JOIN after_clear_v2_events ac
        ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
      JOIN order_events oe
        ON oe.order_hash = c.alice_order_hash AND oe.order_owner = c.alice_order_owner
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
       AND oi.vault_id = c.alice_input_vault_id AND oi.io_type = 'input'
      WHERE lower(c.alice_input_vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
      UNION ALL
      SELECT c.bob_output_vault_id AS vault_id,
             oi.token,
             '-' || ac.bob_output AS amount_change
      FROM clear_v3_events c
      JOIN after_clear_v2_events ac
        ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
      JOIN order_events oe
        ON oe.order_hash = c.bob_order_hash AND oe.order_owner = c.bob_order_owner
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
       AND oi.vault_id = c.bob_output_vault_id AND oi.io_type = 'output'
      WHERE lower(c.bob_output_vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
      UNION ALL
      SELECT c.bob_input_vault_id AS vault_id,
             oi.token,
             ac.bob_input AS amount_change
      FROM clear_v3_events c
      JOIN after_clear_v2_events ac
        ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
      JOIN order_events oe
        ON oe.order_hash = c.bob_order_hash AND oe.order_owner = c.bob_order_owner
      JOIN order_ios oi
        ON oi.transaction_hash = oe.transaction_hash AND oi.log_index = oe.log_index
       AND oi.vault_id = c.bob_input_vault_id AND oi.io_type = 'input'
      WHERE lower(c.bob_input_vault_id) = lower('?vault_id') AND lower(oi.token) = lower('?token')
  ) balance_changes
  GROUP BY vault_id, token
) b
ON d.vault_id = b.vault_id AND d.token = b.token
LEFT JOIN order_ios oi2
ON d.vault_id = oi2.vault_id AND d.token = oi2.token
LEFT JOIN order_events oe2
ON oe2.transaction_hash = oi2.transaction_hash AND oe2.log_index = oi2.log_index
GROUP BY d.vault_id, d.token, d.sender, b.balance
LIMIT 1;

