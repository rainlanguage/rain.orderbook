SELECT
  vd.transaction_hash,
  vd.log_index,
  vd.block_number,
  vd.block_timestamp,
  vd.owner,
  vd.kind            AS change_type,
  vd.token,
  vd.vault_id,
  vd.delta,
  /* cumulative balance up to this row (inclusive) */
  (
    SELECT COALESCE(FLOAT_SUM(vd2.delta),
      FLOAT_ZERO_HEX())
    FROM vault_deltas vd2
    WHERE vd2.owner    = vd.owner
      AND vd2.token    = vd.token
      AND vd2.vault_id = vd.vault_id
      AND (
           vd2.block_number <  vd.block_number
        OR (vd2.block_number = vd.block_number AND vd2.log_index <= vd.log_index)
      )
  ) AS running_balance
FROM (
  SELECT
    '?vault_id' AS vault_id,
    '?token'                         AS token,
    COALESCE(
      /* most recent order touching this (token, vault) */
      (
        SELECT oe.order_owner
        FROM order_ios io
        JOIN order_events oe
          ON oe.transaction_hash = io.transaction_hash
         AND oe.log_index       = io.log_index
        WHERE io.token    = '?token'
          AND io.vault_id = '?vault_id'
        ORDER BY oe.block_number DESC
        LIMIT 1
      ),
      /* fallback to latest depositor/withdrawer */
      (
        SELECT owner FROM (
          SELECT d.sender AS owner, d.block_number
          FROM deposits d
          WHERE d.token    = '?token'
            AND d.vault_id = '?vault_id'
          UNION ALL
          SELECT w.sender AS owner, w.block_number
          FROM withdrawals w
          WHERE w.token    = '?token'
            AND w.vault_id = '?vault_id'
          ORDER BY block_number DESC
          LIMIT 1
        ) AS last_dw
      )
    ) AS owner
) AS o
JOIN vault_deltas vd
  ON vd.vault_id = o.vault_id
 AND vd.token    = o.token
 AND vd.owner    = o.owner
ORDER BY vd.block_number DESC, vd.log_index DESC, vd.kind;
