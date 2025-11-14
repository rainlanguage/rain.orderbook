WITH filtered_vault_deltas AS (
  SELECT vd.*
  FROM vault_deltas vd
  WHERE 1 = 1
    /*INNER_CHAIN_IDS_CLAUSE*/
    /*INNER_ORDERBOOKS_CLAUSE*/
),
vault_balances AS (
  SELECT
    vd.chain_id,
    vd.orderbook_address,
    vd.owner,
    vd.token,
    vd.vault_id,
    COALESCE(
      FLOAT_SUM(vd.delta ORDER BY vd.block_number, vd.log_index),
      FLOAT_ZERO_HEX()
    ) AS balance
  FROM filtered_vault_deltas vd
  WHERE 1 = 1
    /*CHAIN_IDS_CLAUSE*/
    /*ORDERBOOKS_CLAUSE*/
  GROUP BY vd.chain_id, vd.orderbook_address, vd.owner, vd.token, vd.vault_id
)
SELECT
  o.chain_id,
  o.orderbook_address,
  o.vault_id,
  o.token,
  o.owner,
  et.name   AS token_name,
  et.symbol AS token_symbol,
  et.decimals AS token_decimals,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT (
        '0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = o.chain_id
       AND lower(oe.orderbook_address) = lower(o.orderbook_address)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE e1.chain_id = o.chain_id
          AND lower(e1.orderbook_address) = lower(o.orderbook_address)
          AND NOT EXISTS (
            SELECT 1 FROM order_events e2
            WHERE e2.chain_id = o.chain_id
              AND lower(e2.orderbook_address) = lower(o.orderbook_address)
              AND e2.order_owner = e1.order_owner
              AND e2.order_nonce = e1.order_nonce
              AND (
                e2.block_number > e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
              )
          )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = o.chain_id
        AND lower(io.orderbook_address) = lower(o.orderbook_address)
        AND lower(io.token)    = lower(o.token)
        AND lower(io.vault_id) = lower(o.vault_id)
        AND UPPER(io.io_type) = 'INPUT'
      ORDER BY oe.block_number DESC, oe.log_index DESC, oe.order_hash
    ) AS q_in
  ) AS input_orders,
  (
    SELECT GROUP_CONCAT(item)
    FROM (
      SELECT DISTINCT (
        '0x01' || ':' || oe.order_hash || ':' ||
        CASE WHEN l.event_type = 'AddOrderV3' THEN '1' ELSE '0' END
      ) AS item
      FROM order_ios io
      JOIN order_events oe
        ON oe.chain_id = o.chain_id
       AND lower(oe.orderbook_address) = lower(o.orderbook_address)
       AND oe.transaction_hash = io.transaction_hash
       AND oe.log_index       = io.log_index
      JOIN (
        SELECT e1.order_owner, e1.order_nonce, e1.event_type
        FROM order_events e1
        WHERE e1.chain_id = o.chain_id
          AND lower(e1.orderbook_address) = lower(o.orderbook_address)
          AND NOT EXISTS (
            SELECT 1 FROM order_events e2
            WHERE e2.chain_id = o.chain_id
              AND lower(e2.orderbook_address) = lower(o.orderbook_address)
              AND e2.order_owner = e1.order_owner
              AND e2.order_nonce = e1.order_nonce
              AND (
                e2.block_number > e1.block_number
                OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
              )
          )
      ) l ON l.order_owner = oe.order_owner AND l.order_nonce = oe.order_nonce
      WHERE io.chain_id = o.chain_id
        AND lower(io.orderbook_address) = lower(o.orderbook_address)
        AND lower(io.token)    = lower(o.token)
        AND lower(io.vault_id) = lower(o.vault_id)
        AND UPPER(io.io_type) = 'OUTPUT'
      ORDER BY oe.block_number DESC, oe.log_index DESC, oe.order_hash
    ) AS q_out
  ) AS output_orders,
  o.balance
FROM vault_balances o
JOIN erc20_tokens et
  ON et.chain_id = o.chain_id
 AND lower(et.orderbook_address) = lower(o.orderbook_address)
 AND lower(et.token_address) = lower(o.token)
WHERE 1=1
/*OWNERS_CLAUSE*/
/*TOKENS_CLAUSE*/
/*HIDE_ZERO_BALANCE*/
ORDER BY o.chain_id, o.orderbook_address, o.owner, o.token, o.vault_id;
