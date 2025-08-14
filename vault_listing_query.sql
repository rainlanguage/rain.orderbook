SELECT
    d.vault_id,
    d.sender as owner,
    d.token,
    'TODO: balance calculation' as balance,
    GROUP_CONCAT(
        CASE WHEN ios.io_type = 'input' THEN
            ios_order.order_hash
        END
    ) as input_for_orders,
    GROUP_CONCAT(
        CASE WHEN ios.io_type = 'output' THEN
            ios_order.order_hash
        END
    ) as output_for_orders
FROM deposits d
LEFT JOIN order_ios ios ON d.vault_id = ios.vault_id AND d.token = ios.token
LEFT JOIN order_events ios_order ON ios.order_event_id = ios_order.id
WHERE ios_order.event_type = 'add' OR ios_order.event_type IS NULL
GROUP BY d.vault_id, d.sender, d.token
ORDER BY d.vault_id;