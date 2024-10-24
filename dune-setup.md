## Add a New Contract for indexing with Dune 
- Process to add new contracts with Dune for indexing and setting queries for tracking transactions and volumes.
- Process is fairly simple to add the contract and update it with an existing query, queries and dashboard may be updated further with contracts.

### Steps to Add a contract for indexing
- Login to https://dune.com/ 
- Navigate to Library > Contacts.
![alt text](https://github.com/Siddharth2207/image-stash/blob/main/images/DuneSetUp/dune-setup-1.png?raw=true)
- Select Add Contract and fill all the contract details.
![alt text](https://github.com/Siddharth2207/image-stash/blob/main/images/DuneSetUp/dune-setup-2.png?raw=true)
- Next, select the orderbook for which we want to set the indexer.
- Select the takeOrder event, depending upon the contract it may be takeOrderV2 or takeOrderV3 for newer versions.
![alt text](https://github.com/Siddharth2207/image-stash/blob/main/images/DuneSetUp/dune-setup3.png?raw=true)
- Add the following SQL query to the console and click on Run.
```
WITH base_token_volume AS (
  SELECT
    t.block_date,
    t.token_bought_symbol AS token_symbol,
    SUM(t.amount_usd) AS usd_volume
  FROM dex.trades AS t
  INNER JOIN (
    SELECT evt_tx_hash, evt_block_number FROM raindex_base.OrderBook_evt_TakeOrder
    UNION ALL
    SELECT evt_tx_hash, evt_block_number FROM raindex_base.OrderBook_evt_TakeOrderV2
  ) AS to
    ON t.tx_hash = to.evt_tx_hash AND t.block_number = to.evt_block_number
  WHERE
    t.block_date > TRY_CAST('2023-09-01' AS DATE)
  GROUP BY
    t.block_date,
    t.token_bought_symbol
)
SELECT
  block_date,
  token_symbol AS token,
  usd_volume
FROM base_token_volume
ORDER BY
  block_date,
  token_symbol;
```
- Save the query and navigate back to dashboard  and the contract indexing should be visible

