SELECT 
    d.vault_id,
    d.token,
    MAX(oi.decimals) as decimals,
    d.sender as owner,
    COALESCE(b.balance, '0') as balance,
    GROUP_CONCAT(DISTINCT CASE WHEN oi.io_type = 'input' THEN oe.order_hash END) as input_order_hashes,
    GROUP_CONCAT(DISTINCT CASE WHEN oi.io_type = 'output' THEN oe.order_hash END) as output_order_hashes
FROM (
    SELECT DISTINCT vault_id, token, sender
    FROM deposits
) d
LEFT JOIN (
    SELECT vault_id, token, BIGINT_SUM(amount_change) as balance
    FROM (
        SELECT vault_id, token, amount as amount_change FROM deposits
        UNION ALL
        SELECT vault_id, token, '-' || amount as amount_change FROM withdrawals
        UNION ALL
        -- Input vaults receive tokens when orders are taken (+)
        SELECT oi.vault_id, oi.token, t.input_amount as amount_change
        FROM take_orders t
        JOIN order_events oe ON t.order_owner = oe.owner AND t.order_nonce = oe.nonce  
        JOIN order_ios oi ON oe.id = oi.order_event_id 
        WHERE oi.io_index = t.input_io_index AND oi.io_type = 'input'
        UNION ALL
        -- Output vaults send tokens when orders are taken (-)
        SELECT oi.vault_id, oi.token, '-' || t.output_amount as amount_change
        FROM take_orders t
        JOIN order_events oe ON t.order_owner = oe.owner AND t.order_nonce = oe.nonce
        JOIN order_ios oi ON oe.id = oi.order_event_id
        WHERE oi.io_index = t.output_io_index AND oi.io_type = 'output'
        UNION ALL
        -- Clear events: Alice output vault loses tokens
        SELECT c.alice_output_vault_id as vault_id,
               oi.token,
               '-' || ac.alice_output as amount_change
        FROM clear_v2_events c
        JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
        JOIN order_events oe ON c.alice_order_hash = oe.order_hash
        JOIN order_ios oi ON oe.id = oi.order_event_id AND oi.vault_id = c.alice_output_vault_id AND oi.io_type = 'output'
        UNION ALL
        -- Clear events: Alice input vault gains tokens  
        SELECT c.alice_input_vault_id as vault_id,
               oi.token,
               ac.alice_input as amount_change
        FROM clear_v2_events c
        JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
        JOIN order_events oe ON c.alice_order_hash = oe.order_hash
        JOIN order_ios oi ON oe.id = oi.order_event_id AND oi.vault_id = c.alice_input_vault_id AND oi.io_type = 'input'
        UNION ALL
        -- Clear events: Bob output vault loses tokens
        SELECT c.bob_output_vault_id as vault_id,
               oi.token,
               '-' || ac.bob_output as amount_change
        FROM clear_v2_events c
        JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
        JOIN order_events oe ON c.bob_order_hash = oe.order_hash
        JOIN order_ios oi ON oe.id = oi.order_event_id AND oi.vault_id = c.bob_output_vault_id AND oi.io_type = 'output'
        UNION ALL
        -- Clear events: Bob input vault gains tokens
        SELECT c.bob_input_vault_id as vault_id,
               oi.token,
               ac.bob_input as amount_change
        FROM clear_v2_events c
        JOIN after_clear_events ac ON c.transaction_hash = ac.transaction_hash AND c.sender = ac.sender
        JOIN order_events oe ON c.bob_order_hash = oe.order_hash
        JOIN order_ios oi ON oe.id = oi.order_event_id AND oi.vault_id = c.bob_input_vault_id AND oi.io_type = 'input'
    ) balance_changes
    GROUP BY vault_id, token
) b ON d.vault_id = b.vault_id AND d.token = b.token
LEFT JOIN order_ios oi ON d.vault_id = oi.vault_id AND d.token = oi.token
LEFT JOIN order_events oe ON oi.order_event_id = oe.id
GROUP BY d.vault_id, d.token, d.sender, b.balance
ORDER BY d.vault_id, d.token;