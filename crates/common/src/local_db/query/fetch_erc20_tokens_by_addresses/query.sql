SELECT chain_id, orderbook_address, token_address, name, symbol, decimals
FROM erc20_tokens
WHERE chain_id = ?1 AND lower(orderbook_address) = lower(?2)
  /*ADDRESSES_CLAUSE*/;
