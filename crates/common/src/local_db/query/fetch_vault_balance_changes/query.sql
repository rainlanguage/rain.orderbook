WITH
params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS orderbook_address,
    ?3 AS vault_id,
    ?4 AS token,
    ?5 AS owner
),
vault_changes AS (
  SELECT
    vd.transaction_hash,
    vd.log_index,
    vd.block_number,
    vd.block_timestamp,
    vd.owner,
    vd.kind,
    vd.token,
    vd.vault_id,
    vd.delta
  FROM vault_deltas vd
  JOIN params p
    ON vd.chain_id = p.chain_id
   AND vd.orderbook_address = p.orderbook_address
   AND vd.vault_id = p.vault_id
   AND vd.token = p.token
   AND vd.owner = p.owner
),
running_balances AS (
  SELECT
    vc.*,
    (
      SELECT COALESCE(FLOAT_SUM(prev.delta), FLOAT_ZERO_HEX())
      FROM (
        SELECT delta, block_number, log_index
        FROM vault_changes
        WHERE block_number <  vc.block_number
           OR (block_number = vc.block_number AND log_index <= vc.log_index)
        ORDER BY block_number, log_index
      ) AS prev
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
  vc.running_balance,
  vc.running_balance_components
FROM running_balances vc
ORDER BY vc.block_number DESC, vc.log_index DESC, vc.kind;
