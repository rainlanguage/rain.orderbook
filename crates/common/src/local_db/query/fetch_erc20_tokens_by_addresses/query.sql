SELECT chain_id, address, name, symbol, decimals
FROM erc20_tokens
WHERE chain_id = ?1
  /*ADDRESSES_CLAUSE*/;
