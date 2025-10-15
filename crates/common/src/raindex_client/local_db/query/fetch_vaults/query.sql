SELECT
  o.vault_id,
  o.token,
  o.owner,
  '0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB' AS orderbook_address,
  (
    SELECT GROUP_CONCAT(order_hash)
    FROM (
      SELECT DISTINCT oe.order_hash
      FROM order_ios io
      JOIN order_events oe
        ON oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      WHERE io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'INPUT'
      ORDER BY oe.order_hash
    ) AS q_in
  ) AS input_order_hashes,
  (
    SELECT GROUP_CONCAT(order_hash)
    FROM (
      SELECT DISTINCT oe.order_hash
      FROM order_ios io
      JOIN order_events oe
        ON oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      WHERE io.token    = o.token
        AND io.vault_id = o.vault_id
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.order_hash
    ) AS q_out
  ) AS output_order_hashes,
  COALESCE((
    SELECT FLOAT_SUM(vd.delta)
    FROM vault_deltas vd
    WHERE vd.owner    = o.owner
      AND vd.token    = o.token
      AND vd.vault_id = o.vault_id
  ), '0x0000000000000000000000000000000000000000000000000000000000000000') AS balance
FROM (
  /* all distinct (owner, token, vault_id) that ever had a delta */
  SELECT DISTINCT owner, token, vault_id
  FROM vault_deltas
) AS o
ORDER BY o.owner, o.token, o.vault_id;
