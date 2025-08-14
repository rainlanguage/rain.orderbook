SELECT
      add_events.order_hash,
      add_events.owner,
      add_events.block_timestamp as creation_time,
      GROUP_CONCAT(
          CASE WHEN ios.io_type = 'input' THEN
              ios.token || ':' || ios.decimals || ':' || ios.vault_id
          END
      ) as inputs,
      GROUP_CONCAT(
          CASE WHEN ios.io_type = 'output' THEN
              ios.token || ':' || ios.decimals || ':' || ios.vault_id
          END
      ) as outputs,
      (
          SELECT COUNT(*)
          FROM take_orders
          WHERE order_owner = add_events.owner
            AND order_nonce = add_events.nonce
      ) as trade_count
  FROM order_events as add_events
  LEFT JOIN order_ios as ios ON add_events.id = ios.order_event_id
  WHERE add_events.event_type = 'add'
    AND add_events.order_hash NOT IN (
        SELECT order_hash
        FROM order_events
        WHERE event_type = 'remove'
    )
  GROUP BY add_events.order_hash, add_events.owner, add_events.block_timestamp,
  add_events.nonce
  ORDER BY add_events.block_timestamp DESC;