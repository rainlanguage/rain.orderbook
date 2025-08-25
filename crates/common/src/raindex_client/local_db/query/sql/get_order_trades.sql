SELECT
    datetime(t.block_timestamp, 'unixepoch') as trade_date,
    -- t.sender, this requires us to fetch the tx sender for every transaction. not sure if we should do that
    t.transaction_hash,
    t.input_amount,
    t.output_amount
FROM order_events o
JOIN take_orders t ON o.owner = t.order_owner 
    AND o.nonce = t.order_nonce
WHERE o.order_hash = ?
    AND o.event_type = 'add'
ORDER BY t.block_timestamp DESC;