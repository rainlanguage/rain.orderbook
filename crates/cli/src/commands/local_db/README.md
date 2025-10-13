# Rain Orderbook CLI — LocalDB

This folder hosts commands that help you build and manage a local SQLite database for Raindex Orderbook events.

The typical pipeline is:

1) Fetch raw events from chain → JSON
2) Decode events → JSON
3) Generate SQL for tokens (metadata) and events → SQL
4) Create a SQLite DB and dump it (optional)

Notes
- Run commands from the repo root. If your default `cargo run` package is not the CLI, add `-p rain_orderbook_cli`.
- Paths below reference this folder. Adjust to your local paths as needed.

## Incremental Sync (fetch, decode, insert)

`local-db sync` is now a single command that mirrors the browser sync logic. It accepts the Rain settings YAML directly, resolves the primary orderbook for the given chain, and then updates your SQLite database by fetching the missing on-chain events, decoding them, preparing any missing token metadata, and applying the combined SQL transaction.

Example:

```bash
cargo run -p rain_orderbook_cli -- local-db sync \
  --db-path "./data/orderbook.db" \
  --chain-id 42161 \
  --settings-yaml "..." \
  --api-token hyper-token \
  --start-block 352866209 \
  --end-block 352999999
```

Key behaviour:
- The command parses the provided settings YAML to discover the orderbook deployment. No manual `--orderbook-address`, `--deployment-block`, or user-supplied RPC URLs are required.
- HyperRPC log fetching uses the provided `--api-token`; ERC-20 metadata reads reuse the network RPCs declared in settings. Start/end blocks are optional—when omitted the runner resumes from the last synced block and stops at the chain head.
- When the DB is empty the schema is created automatically, `sync_status` is advanced to the synced block, and existing token metadata is reused while missing addresses trigger live fetches.

## Fetch Events

Fetch OrderBook events from a chain and write them to a JSON file.

Example:

```bash
cargo run local-db fetch-events \
  --api-token "some-token" \
  --chain-id 42161 \
  --start-block 352866209 \
  --orderbook-address "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB" \
  --output-file "src/commands/local_db/events.json"
```

If `--end-block` is omitted, the latest block is used.

## Decode Events

Decode the raw logs output into normalized JSON with `event_type` and `decoded_data`.

Example:

```bash
cargo run local-db decode-events \
  --input-file  "src/commands/local_db/events.json" \
  --output-file "src/commands/local_db/decoded_events.json"
```

## Fetch Token Metadata (tokens.json)

Scan decoded events for token addresses, fetch ERC20 metadata via RPC, and write `tokens.json`.

Example:

```bash
cargo run local-db tokens-fetch \
  --rpc         "https://arbitrum-one-rpc.publicnode.com" \
  --input-file  "src/commands/local_db/decoded_events.json" \
  --output-file "src/commands/local_db/tokens.json"
```

## Generate Token SQL from tokens.json

Turn `tokens.json` into an upsert SQL for `erc20_tokens`.

Example:

```bash
cargo run local-db tokens-to-sql \
  --chain-id    42161 \
  --input-file  "src/commands/local_db/tokens.json" \
  --output-file "src/commands/local_db/tokens.sql"
```

## Generate SQL Commands from Decoded Events

Builds all event INSERT statements and updates `sync_status`. This command requires token decimals to compute deposit amounts, so you must provide `--tokens-file tokens.json`. This command only generates event SQL; use `tokens-to-sql` for token upserts.

Example (using tokens.json, no RPC):

```bash
cargo run local-db decoded-events-to-sql \
  --chain-id    42161 \
  --tokens-file "src/commands/local_db/tokens.json" \
  --input-file  "src/commands/local_db/decoded_events.json" \
  --output-file "src/commands/local_db/events.sql" \
  --end-block   373116382
```


Output is a single SQL transaction containing all event INSERTs (deposits, withdrawals, orders, takes, clears, meta) and the `sync_status` update. It does not include token upserts — generate those with `tokens-to-sql`.

## Create Database and Dump

Create a SQLite database, apply schema and data SQL, and then dump it (also gzipped).

Example:

```bash
cargo run local-db dump \
  --data-sql "src/commands/local_db/tokens.sql" \
  --data-sql "src/commands/local_db/events.sql" \
  --table-schema-file "../../crates/common/src/raindex_client/local_db/query/create_tables/query.sql" \
  --end-block 373116382
```

This produces `local_db_<end_block>.db`, `local_db_<end_block>.sql`, and `local_db_<end_block>.sql.gz` in this folder by default.

All SQL view definitions in `crates/common/src/raindex_client/local_db/views` are loaded automatically before the dump so the exported schema includes them.

## Tips
- Recommended pipeline: Fetch Events → Decode → Tokens (tokens-fetch, tokens-to-sql) → Events (decoded-events-to-sql with --tokens-file) → Dump.
- You can skip tokens-fetch and pass `--rpc` to decoded-events-to-sql for decimals, but you still need to run `tokens-to-sql` to produce token upserts for the DB.
