SELECT
  l.chain_id AS chain_id,
  COALESCE(la.order_hash, l.order_hash) AS order_hash,
  l.order_owner AS owner,
  fa.block_timestamp AS block_timestamp,
  fa.block_number AS block_number,
  l.orderbook_address AS orderbook_address,
  la.order_bytes AS order_bytes,
  json_group_array(
    CASE
      WHEN lower(ios.io_type) = 'input'
      THEN json_object(
        'ioIndex', ios.io_index,
        'vault', json_object(
          'vaultId', ios.vault_id,
          'token', ios.token,
          'owner', COALESCE(vo.owner, lower(l.order_owner)),
          'orderbookAddress', l.orderbook_address,
          'tokenName', COALESCE(tok.name, ''),
          'tokenSymbol', COALESCE(tok.symbol, ''),
          'tokenDecimals', COALESCE(tok.decimals, 0),
          'balance', COALESCE(vb.balance_hex, FLOAT_ZERO_HEX()),
          'inputOrders', NULL,
          'outputOrders', NULL
        )
      )
    END
  ) AS inputs,
  json_group_array(
    CASE
      WHEN lower(ios.io_type) = 'output'
      THEN json_object(
        'ioIndex', ios.io_index,
        'vault', json_object(
          'vaultId', ios.vault_id,
          'token', ios.token,
          'owner', COALESCE(vo.owner, lower(l.order_owner)),
          'orderbookAddress', l.orderbook_address,
          'tokenName', COALESCE(tok.name, ''),
          'tokenSymbol', COALESCE(tok.symbol, ''),
          'tokenDecimals', COALESCE(tok.decimals, 0),
          'balance', COALESCE(vb.balance_hex, FLOAT_ZERO_HEX()),
          'inputOrders', NULL,
          'outputOrders', NULL
        )
      )
    END
  ) AS outputs,
  COALESCE(tc.trade_count, 0) + COALESCE(cc.trade_count, 0) AS trade_count,
  (l.event_type = 'AddOrderV3') AS active,
  la.transaction_hash AS transaction_hash,
  (
    SELECT m.meta
    FROM meta_events m
    WHERE m.chain_id = l.chain_id
      AND lower(m.orderbook_address) = lower(l.orderbook_address)
      AND lower(m.subject) = lower(COALESCE(la.order_hash, l.order_hash))
    ORDER BY m.block_number DESC, m.log_index DESC
    LIMIT 1
  ) AS meta
FROM (
  SELECT
    latest.chain_id,
    latest.orderbook_address,
    latest.order_owner,
    latest.order_nonce,
    latest.order_hash,
    latest.transaction_hash,
    latest.log_index,
    latest.block_number,
    latest.block_timestamp,
    latest.event_type
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.order_hash,
      oe.transaction_hash,
      oe.log_index,
      oe.block_number,
      oe.block_timestamp,
      oe.event_type,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          lower(oe.orderbook_address),
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number DESC, oe.log_index DESC
      ) AS row_rank_latest
    FROM order_events oe
    WHERE 1 = 1
      /*MAIN_CHAIN_IDS_CLAUSE*/
      /*MAIN_ORDERBOOKS_CLAUSE*/
  ) latest
  WHERE latest.row_rank_latest = 1
) l
LEFT JOIN (
  SELECT
    ranked.chain_id,
    ranked.orderbook_address,
    ranked.order_owner,
    ranked.order_nonce,
    ranked.transaction_hash,
    ranked.log_index,
    ranked.order_hash,
    ranked.order_bytes
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.transaction_hash,
      oe.log_index,
      oe.order_hash,
      oe.order_bytes,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          lower(oe.orderbook_address),
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number DESC, oe.log_index DESC
      ) AS row_rank_add
    FROM order_events oe
    WHERE oe.event_type = 'AddOrderV3'
      /*LATEST_ADD_CHAIN_IDS_CLAUSE*/
      /*LATEST_ADD_ORDERBOOKS_CLAUSE*/
  ) ranked
  WHERE ranked.row_rank_add = 1
) la
  ON la.chain_id = l.chain_id
 AND lower(la.orderbook_address) = lower(l.orderbook_address)
 AND la.order_owner = l.order_owner
 AND la.order_nonce = l.order_nonce
LEFT JOIN order_ios ios
  ON ios.chain_id = l.chain_id
 AND lower(ios.orderbook_address) = lower(l.orderbook_address)
 AND ios.transaction_hash = la.transaction_hash
 AND ios.log_index = la.log_index
LEFT JOIN erc20_tokens tok
  ON tok.chain_id = ios.chain_id
 AND lower(tok.orderbook_address) = lower(ios.orderbook_address)
 AND lower(tok.token_address) = lower(ios.token)
LEFT JOIN (
  SELECT
    chain_id,
    lower(orderbook_address) AS orderbook_address,
    lower(token) AS token,
    lower(vault_id) AS vault_id,
    substr(MAX(owner_key), 33) AS owner
  FROM (
    SELECT
      io.chain_id,
      io.orderbook_address,
      io.token,
      io.vault_id,
      printf('%020d:%010d:%s', oe.block_number, oe.log_index, lower(oe.order_owner)) AS owner_key
    FROM order_ios io
    JOIN order_events oe
      ON oe.chain_id = io.chain_id
     AND lower(oe.orderbook_address) = lower(io.orderbook_address)
     AND oe.transaction_hash = io.transaction_hash
     AND oe.log_index = io.log_index
    WHERE UPPER(io.io_type) IN ('INPUT', 'OUTPUT')
    UNION ALL
    SELECT
      d.chain_id,
      d.orderbook_address,
      d.token,
      d.vault_id,
      printf('%020d:%010d:%s', d.block_number, d.log_index, lower(d.sender)) AS owner_key
    FROM deposits d
    UNION ALL
    SELECT
      w.chain_id,
      w.orderbook_address,
      w.token,
      w.vault_id,
      printf('%020d:%010d:%s', w.block_number, w.log_index, lower(w.sender)) AS owner_key
    FROM withdrawals w
  )
  GROUP BY chain_id, lower(orderbook_address), lower(token), lower(vault_id)
) vo
  ON vo.chain_id = ios.chain_id
 AND vo.orderbook_address = lower(ios.orderbook_address)
 AND vo.token = lower(ios.token)
 AND vo.vault_id = lower(ios.vault_id)
LEFT JOIN (
  SELECT
    vd.chain_id,
    lower(vd.orderbook_address) AS orderbook_address,
    lower(vd.token) AS token,
    lower(vd.vault_id) AS vault_id,
    FLOAT_SUM(vd.delta) AS balance_hex
  FROM vault_deltas vd
  GROUP BY vd.chain_id, lower(vd.orderbook_address), lower(vd.token), lower(vd.vault_id)
) vb
  ON vb.chain_id = ios.chain_id
 AND vb.orderbook_address = lower(ios.orderbook_address)
 AND vb.token = lower(ios.token)
 AND vb.vault_id = lower(ios.vault_id)
LEFT JOIN (
  SELECT
    t.chain_id,
    lower(t.orderbook_address) AS orderbook_address,
    t.order_owner,
    t.order_nonce,
    COUNT(*) AS trade_count
  FROM take_orders t
  WHERE 1 = 1
    /*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/
    /*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/
  GROUP BY t.chain_id, lower(t.orderbook_address), t.order_owner, t.order_nonce
) tc
  ON tc.chain_id = l.chain_id
 AND tc.orderbook_address = lower(l.orderbook_address)
 AND tc.order_owner = l.order_owner
 AND tc.order_nonce = l.order_nonce
LEFT JOIN (
  SELECT
    entries.chain_id,
    lower(entries.orderbook_address) AS orderbook_address,
    lower(entries.order_hash) AS order_hash,
    COUNT(*) AS trade_count
  FROM (
    SELECT
      c.chain_id,
      c.orderbook_address,
      c.alice_order_hash AS order_hash
    FROM clear_v3_events c
    WHERE c.alice_order_hash IS NOT NULL
    UNION ALL
    SELECT
      c.chain_id,
      c.orderbook_address,
      c.bob_order_hash AS order_hash
    FROM clear_v3_events c
    WHERE c.bob_order_hash IS NOT NULL
  ) entries
  WHERE entries.order_hash IS NOT NULL
    /*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/
    /*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/
  GROUP BY entries.chain_id, lower(entries.orderbook_address), lower(entries.order_hash)
) cc
  ON cc.chain_id = l.chain_id
 AND cc.orderbook_address = lower(l.orderbook_address)
 AND cc.order_hash = lower(COALESCE(la.order_hash, l.order_hash))
LEFT JOIN (
  SELECT
    ranked.chain_id,
    ranked.orderbook_address,
    ranked.order_owner,
    ranked.order_nonce,
    ranked.block_timestamp,
    ranked.block_number
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.block_timestamp,
      oe.block_number,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          lower(oe.orderbook_address),
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number ASC, oe.log_index ASC
      ) AS row_rank_first_add
    FROM order_events oe
    WHERE oe.event_type = 'AddOrderV3'
      /*FIRST_ADD_CHAIN_IDS_CLAUSE*/
      /*FIRST_ADD_ORDERBOOKS_CLAUSE*/
  ) ranked
  WHERE ranked.row_rank_first_add = 1
) fa
  ON fa.chain_id = l.chain_id
 AND lower(fa.orderbook_address) = lower(l.orderbook_address)
 AND fa.order_owner = l.order_owner
 AND fa.order_nonce = l.order_nonce
WHERE
  (
    ?1 = 'all'
    OR (?1 = 'active' AND l.event_type = 'AddOrderV3')
    OR (?1 = 'inactive' AND l.event_type = 'RemoveOrderV3')
  )
/*OWNERS_CLAUSE*/
/*ORDER_HASH_CLAUSE*/
/*TOKENS_CLAUSE*/
GROUP BY
  l.chain_id,
  COALESCE(la.order_hash, l.order_hash),
  l.order_owner,
  fa.block_timestamp,
  fa.block_number,
  l.orderbook_address,
  l.order_nonce,
  l.event_type,
  la.transaction_hash
ORDER BY fa.block_timestamp DESC;
