SELECT DISTINCT
    chain_id AS chainId,
    orderbook_address AS orderbookAddress,
    token_address AS tokenAddress,
    name,
    symbol,
    decimals
FROM erc20_tokens
WHERE 1=1
  /*CHAIN_IDS_CLAUSE*/
  /*ORDERBOOKS_CLAUSE*/
ORDER BY chain_id, orderbook_address, token_address;

