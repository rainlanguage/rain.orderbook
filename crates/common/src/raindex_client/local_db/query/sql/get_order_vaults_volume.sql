SELECT
    ios.vault_id,
    ios.token,
    ios.decimals,
    COALESCE(SUM(CASE 
        WHEN ios.io_type = 'input' THEN CAST(t.output_amount AS REAL)
        ELSE 0 
    END), 0) as total_in,
    COALESCE(SUM(CASE 
        WHEN ios.io_type = 'output' THEN CAST(t.input_amount AS REAL)
        ELSE 0 
    END), 0) as total_out,
    ABS(COALESCE(SUM(CASE 
        WHEN ios.io_type = 'input' THEN CAST(t.output_amount AS REAL)
        ELSE 0 
    END), 0) - COALESCE(SUM(CASE 
        WHEN ios.io_type = 'output' THEN CAST(t.input_amount AS REAL)
        ELSE 0 
    END), 0)) as net_volume,
    COALESCE(SUM(CASE 
        WHEN ios.io_type = 'input' THEN CAST(t.output_amount AS REAL)
        ELSE 0 
    END), 0) + COALESCE(SUM(CASE 
        WHEN ios.io_type = 'output' THEN CAST(t.input_amount AS REAL)
        ELSE 0 
    END), 0) as total_volume
FROM order_events o
JOIN order_ios ios ON o.id = ios.order_event_id
LEFT JOIN take_orders t ON o.owner = t.order_owner 
    AND o.nonce = t.order_nonce
    AND ((ios.io_type = 'input' AND ios.io_index = t.input_io_index)
         OR (ios.io_type = 'output' AND ios.io_index = t.output_io_index))
WHERE o.order_hash = ?
    AND o.event_type = 'add'
GROUP BY ios.vault_id, ios.token, ios.decimals
ORDER BY ios.vault_id;