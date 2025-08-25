-- Separate query for vault balance calculation
-- Sum deposits minus withdrawals for each vault
SELECT 
    vault_id,
    token,
    SUM(
        CASE 
            WHEN event_type = 'deposit' THEN CAST(amount AS INTEGER)
            WHEN event_type = 'withdrawal' THEN -CAST(amount AS INTEGER)
        END
    ) as balance
FROM (
    SELECT vault_id, token, amount, 'deposit' as event_type FROM deposits
    UNION ALL
    SELECT vault_id, token, amount, 'withdrawal' as event_type FROM withdrawals
) vault_movements
GROUP BY vault_id, token
ORDER BY vault_id;