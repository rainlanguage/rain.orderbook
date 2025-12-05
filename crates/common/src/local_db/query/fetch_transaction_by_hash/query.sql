WITH params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS orderbook_address,
    ?3 AS transaction_hash
)
SELECT
  vbc.transaction_hash AS transactionHash,
  vbc.block_number AS blockNumber,
  vbc.block_timestamp AS blockTimestamp,
  vbc.owner
FROM vault_balance_changes vbc
JOIN params p
  ON p.chain_id = vbc.chain_id
 AND p.orderbook_address = vbc.orderbook_address
 AND p.transaction_hash = vbc.transaction_hash
ORDER BY vbc.log_index ASC
LIMIT 1;

