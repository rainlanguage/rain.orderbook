SELECT
  vd.chain_id,
  vd.orderbook_address,
  vd.transaction_hash,
  vd.log_index,
  vd.block_number,
  vd.block_timestamp,
  vd.owner,
  vd.kind            AS change_type,
  vd.token,
  vd.vault_id,
  vd.delta,
  (
    SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
    FROM vault_deltas vd2
    WHERE vd2.chain_id = vd.chain_id
      AND lower(vd2.orderbook_address) = lower(vd.orderbook_address)
      AND vd2.owner    = vd.owner
      AND vd2.token    = vd.token
      AND vd2.vault_id = vd.vault_id
      AND (
           vd2.block_number <  vd.block_number
        OR (vd2.block_number = vd.block_number AND vd2.log_index <= vd.log_index)
      )
  ) AS running_balance
FROM (
  SELECT
    ?chain_id AS chain_id,
    lower('?orderbook_address') AS orderbook_address,
    '?vault_id' AS vault_id,
    '?token' AS token,
    COALESCE(
      (
        SELECT oe.order_owner
        FROM order_ios io
        JOIN order_events oe
          ON oe.chain_id = io.chain_id
         AND lower(oe.orderbook_address) = lower(io.orderbook_address)
         AND oe.transaction_hash = io.transaction_hash
         AND oe.log_index = io.log_index
        WHERE io.chain_id = ?chain_id
          AND lower(io.orderbook_address) = lower('?orderbook_address')
          AND io.token = '?token'
          AND io.vault_id = '?vault_id'
        ORDER BY oe.block_number DESC, oe.log_index DESC
        LIMIT 1
      ),
      (
        SELECT owner FROM (
          SELECT d.sender AS owner, d.block_number, d.log_index
          FROM deposits d
          WHERE d.chain_id = ?chain_id
            AND lower(d.orderbook_address) = lower('?orderbook_address')
            AND d.token = '?token'
            AND d.vault_id = '?vault_id'
          UNION ALL
          SELECT w.sender AS owner, w.block_number, w.log_index
          FROM withdrawals w
          WHERE w.chain_id = ?chain_id
            AND lower(w.orderbook_address) = lower('?orderbook_address')
            AND w.token = '?token'
            AND w.vault_id = '?vault_id'
          ORDER BY block_number DESC, log_index DESC
          LIMIT 1
        )
      )
    ) AS owner
) AS os
JOIN vault_deltas vd
  ON vd.chain_id = os.chain_id
 AND lower(vd.orderbook_address) = os.orderbook_address
 AND vd.vault_id = os.vault_id
 AND vd.token    = os.token
 AND vd.owner    = os.owner
ORDER BY vd.block_number DESC, vd.log_index DESC, vd.kind;
