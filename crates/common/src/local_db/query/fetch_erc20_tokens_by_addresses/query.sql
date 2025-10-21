SELECT chain_id, address, name, symbol, decimals
FROM erc20_tokens
WHERE chain_id = ?chain_id
  AND address IN (?addresses_in);

