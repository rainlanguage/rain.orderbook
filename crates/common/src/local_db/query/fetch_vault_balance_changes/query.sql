WITH params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS orderbook_address,
    ?3 AS vault_id,
    ?4 AS token,
    ?5 AS owner
)
SELECT
  vbc.transaction_hash AS transactionHash,
  vbc.log_index AS logIndex,
  vbc.block_number AS blockNumber,
  vbc.block_timestamp AS blockTimestamp,
  vbc.owner,
  vbc.change_type AS changeType,
  vbc.token,
  vbc.vault_id AS vaultId,
  vbc.delta,
  (
    SELECT COALESCE(
      FLOAT_SUM(prev.delta ORDER BY prev.block_number, prev.log_index),
      FLOAT_ZERO_HEX()
    )
    FROM vault_balance_changes prev
    WHERE prev.chain_id = vbc.chain_id
      AND prev.orderbook_address = vbc.orderbook_address
      AND prev.owner = vbc.owner
      AND prev.token = vbc.token
      AND prev.vault_id = vbc.vault_id
      AND (
           prev.block_number < vbc.block_number
        OR (prev.block_number = vbc.block_number AND prev.log_index <= vbc.log_index)
      )
  ) AS runningBalance
FROM vault_balance_changes vbc
JOIN params p
  ON p.chain_id = vbc.chain_id
 AND p.orderbook_address = vbc.orderbook_address
 AND p.vault_id = vbc.vault_id
 AND p.token = vbc.token
 AND p.owner = vbc.owner
ORDER BY vbc.block_timestamp DESC, vbc.block_number DESC, vbc.log_index DESC;
