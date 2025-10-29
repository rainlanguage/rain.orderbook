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
    SELECT COALESCE(FLOAT_SUM(vd2.delta), FLOAT_ZERO_HEX())
    FROM vault_deltas vd2
    WHERE vd2.chain_id = ?1
      AND lower(vd2.orderbook_address) = lower(?2)
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
    ?3 AS vault_id,
    ?4 AS token,
    COALESCE(
      /* most recent order touching this (token, vault) */
      (
        SELECT oe.order_owner
        FROM order_ios io
        JOIN order_events oe
          ON oe.chain_id = ?1
         AND lower(oe.orderbook_address) = lower(?2)
         AND oe.transaction_hash = io.transaction_hash
         AND oe.log_index       = io.log_index
        WHERE io.chain_id = ?1
          AND lower(io.orderbook_address) = lower(?2)
          AND lower(io.token)    = lower(?4)
          AND lower(io.vault_id) = lower(?3)
        ORDER BY oe.block_number DESC, oe.log_index DESC
        LIMIT 1
      ),
      /* fallback to latest depositor/withdrawer */
      (
        SELECT owner FROM (
          SELECT d.sender AS owner, d.block_number, d.log_index
          FROM deposits d
          WHERE d.chain_id = ?1
            AND lower(d.orderbook_address) = lower(?2)
            AND lower(d.token)    = lower(?4)
            AND lower(d.vault_id) = lower(?3)
          UNION ALL
          SELECT w.sender AS owner, w.block_number, w.log_index
          FROM withdrawals w
          WHERE w.chain_id = ?1
            AND lower(w.orderbook_address) = lower(?2)
            AND lower(w.token)    = lower(?4)
            AND lower(w.vault_id) = lower(?3)
          ORDER BY block_number DESC, log_index DESC
          LIMIT 1
        ) AS last_dw
      )
    ) AS owner
) AS o
JOIN vault_deltas vd
  ON vd.chain_id = ?1
 AND lower(vd.orderbook_address) = lower(?2)
 AND vd.vault_id = o.vault_id
 AND vd.token    = o.token
 AND vd.owner    = o.owner
ORDER BY vd.block_number DESC, vd.log_index DESC, vd.kind;
