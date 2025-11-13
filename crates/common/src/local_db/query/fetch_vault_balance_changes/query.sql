WITH
params AS (
  SELECT
    ?1         AS chain_id,
    lower(?2)  AS orderbook_address,
    lower(?3)  AS vault_id,
    lower(?4)  AS token
),
order_owner AS (
  SELECT oe.order_owner AS owner
  FROM params p
  JOIN order_ios io
    ON io.chain_id = p.chain_id
   AND lower(io.orderbook_address) = p.orderbook_address
   AND lower(io.token)            = p.token
   AND lower(io.vault_id)         = p.vault_id
  JOIN order_events oe
    ON oe.chain_id = p.chain_id
   AND lower(oe.orderbook_address) = p.orderbook_address
   AND oe.transaction_hash = io.transaction_hash
   AND oe.log_index        = io.log_index
  ORDER BY oe.block_number DESC, oe.log_index DESC
  LIMIT 1
),
deposit_withdraw_owner AS (
  SELECT owner
  FROM (
    SELECT d.sender AS owner, d.block_number, d.log_index
    FROM params p
    JOIN deposits d
      ON d.chain_id = p.chain_id
     AND lower(d.orderbook_address) = p.orderbook_address
     AND lower(d.token)             = p.token
     AND lower(d.vault_id)          = p.vault_id
    UNION ALL
    SELECT w.sender AS owner, w.block_number, w.log_index
    FROM params p
    JOIN withdrawals w
      ON w.chain_id = p.chain_id
     AND lower(w.orderbook_address) = p.orderbook_address
     AND lower(w.token)             = p.token
     AND lower(w.vault_id)          = p.vault_id
    ORDER BY block_number DESC, log_index DESC
    LIMIT 1
  ) AS last_dw
),
delta_owner AS (
  SELECT vd.owner
  FROM params p
  JOIN vault_deltas vd
    ON vd.chain_id = p.chain_id
   AND lower(vd.orderbook_address) = p.orderbook_address
   AND lower(vd.vault_id)          = p.vault_id
   AND lower(vd.token)             = p.token
  ORDER BY vd.block_number DESC, vd.log_index DESC
  LIMIT 1
),
resolved_owner AS (
  SELECT
    p.chain_id,
    p.orderbook_address,
    p.vault_id,
    p.token,
    COALESCE(
      (SELECT owner FROM order_owner),
      (SELECT owner FROM deposit_withdraw_owner),
      (SELECT owner FROM delta_owner)
    ) AS owner
  FROM params p
),
vault_changes AS (
  SELECT
    vd.transaction_hash,
    vd.log_index,
    vd.block_number,
    vd.block_timestamp,
    COALESCE(r.owner, vd.owner) AS owner,
    vd.kind,
    vd.token,
    vd.vault_id,
    vd.delta
  FROM vault_deltas vd
  JOIN params p
    ON vd.chain_id = p.chain_id
   AND lower(vd.orderbook_address) = p.orderbook_address
   AND lower(vd.vault_id)          = p.vault_id
   AND lower(vd.token)             = p.token
  LEFT JOIN resolved_owner r
    ON r.chain_id = vd.chain_id
   AND r.orderbook_address = p.orderbook_address
   AND r.vault_id = p.vault_id
   AND r.token = p.token
),
running_balances AS (
  SELECT
    vc.*,
    (
      SELECT COALESCE(FLOAT_SUM(ordered.delta), FLOAT_ZERO_HEX())
      FROM (
        SELECT prev.delta
        FROM vault_changes prev
        WHERE prev.block_number <  vc.block_number
           OR (prev.block_number = vc.block_number AND prev.log_index <= vc.log_index)
        ORDER BY prev.block_number, prev.log_index
      ) AS ordered
    ) AS running_balance,
    (
      SELECT json_group_array(entry)
      FROM (
        SELECT json_object(
          'transaction_hash', prev.transaction_hash,
          'block_number',     prev.block_number,
          'log_index',        prev.log_index,
          'kind',             prev.kind,
          'delta',            prev.delta
        ) AS entry
        FROM vault_changes prev
        WHERE prev.block_number <  vc.block_number
           OR (prev.block_number = vc.block_number AND prev.log_index <= vc.log_index)
        ORDER BY prev.block_number, prev.log_index
      )
    ) AS running_balance_components
  FROM vault_changes vc
)
SELECT
  vc.transaction_hash,
  vc.log_index,
  vc.block_number,
  vc.block_timestamp,
  vc.owner,
  vc.kind AS change_type,
  vc.token,
  vc.vault_id,
  vc.delta,
  /* cumulative balance up to this row (inclusive) */
  vc.running_balance,
  /* ordered list of deltas contributing to the running balance */
  vc.running_balance_components
FROM running_balances vc
ORDER BY vc.block_number DESC, vc.log_index DESC, vc.kind;
