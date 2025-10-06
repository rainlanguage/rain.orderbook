# rain_orderbook_quote — Architecture & Design

This crate provides all functionality to quote Rain Orderbook orders from Rust, expose the API to WASM/TypeScript, and ship a small CLI for ad‑hoc quoting and debugging. It stitches together three external systems:

- On‑chain Orderbook V5 contracts (`IOrderBookV5`) via JSON‑RPC
- The Orderbook subgraph for fetching order details by id
- Rain error decoding to turn revert data into human readable errors

It also includes a debugger that can fork an EVM and trace a `quote2` call for step‑by‑step analysis of interpreter execution.


## Targets and Build Modes

- Library: `rain_orderbook_quote` (default). Always built.
- Binary: uses `src/main.rs` which calls the crate CLI entry `cli::main()`.
- WASM: many types derive `Tsify` and use `wasm-bindgen-utils` helpers. The `cli` and `quote_debug` modules are disabled for wasm via `#[cfg(not(target_family = "wasm"))]`.

Key gating:
- `src/lib.rs` re‑exports modules and conditionally exposes `cli` and `quote_debug` when not targeting wasm.
- `Cargo.toml` has extra `tokio` features for wasm (no full runtime) and enables `rain-interpreter-eval` only for native targets.


## High‑Level Responsibilities

- Build quoting requests for one or many orders (direct or via subgraph lookups).
- Perform efficient batched on‑chain calls using Multicall3 `aggregate3`.
- Represent quote results in a consistent, serializable format (`OrderQuoteValue`).
- Decode revert data (including Rain errors) into structured failures (`FailedQuote`).
- Provide a CLI to drive the above with ergonomic input formats and JSON output.
- Provide a quote debugger that forks an EVM and returns interpreter traces for `quote2` calls.


## Core Data Model

- `OrderQuoteValue`
  - Fields: `max_output: Float`, `ratio: Float`
  - Converts from the ABI type `quote2Return { exists, outputMax, ioRatio }`.
  - Used by both native and wasm; serializes to camelCase.

- `QuoteTarget`
  - A fully specified quote request with the exact `QuoteV2` (includes the `OrderV4` bytes, input/output indices, and optional signed context) and the `orderbook` address.
  - Helpers:
    - `get_order_hash()`: `keccak256(order.abi_encode())`
    - `get_id()`: subgraph order id = `keccak256(orderbook || order_hash)`
    - `validate()`: verifies the input/output indices are in bounds for the given order.
    - `do_quote(rpcs, block_number, gas, multicall_address)`: calls `rpc::batch_quote` with just this target and returns its `QuoteResult`.

- `BatchQuoteTarget(Vec<QuoteTarget>)`
  - Adds `do_quote(...) -> Result<Vec<QuoteResult>, Error>` and is the standard input to RPC batch quoting.

- `QuoteSpec`
  - A quote specification that references an order by `order_hash` and `orderbook` without embedding the order bytes. Includes `input_io_index`, `output_io_index`, and optional `signed_context`.
  - `get_id()`: same subgraph id as above using `orderbook || order_hash`.
  - `get_quote_target_from_subgraph(subgraph_url)`: queries the subgraph to fetch `orderBytes` and builds a `QuoteTarget`.
  - `do_quote(subgraph_url, rpcs, block_number, gas, multicall_address)`: fetches order bytes first, then quotes once.

- `BatchQuoteSpec(Vec<QuoteSpec>)`
  - `get_batch_quote_target_from_subgraph(subgraph_url) -> Vec<Option<QuoteTarget>>`: batch fetches order bytes; returns `None` for missing orders.
  - `do_quote(...) -> Vec<QuoteResult>`: fetches all targets that exist, quotes them in batch, and returns the results aligned to the original spec order; missing specs yield `Err(FailedQuote::NonExistent)`.

- `QuoteResult`
  - Type alias: `Result<OrderQuoteValue, FailedQuote>`.
  - For wasm consumers a matching TS type alias is emitted: `export type QuoteResult = OrderQuoteValue | string`.

- `Pair` and `BatchOrderQuotesResponse` (in `order_quotes.rs`)
  - `Pair` describes a human‑readable input/output pair on an order (`pair_name`, `input_index`, `output_index`).
  - `BatchOrderQuotesResponse` bundles the pair, `block_number`, success flag, optional `data` (`OrderQuoteValue`), and optional string `error`.


## Error Types and Semantics

- `FailedQuote` represents a per‑call failure returned as an element within `Vec<QuoteResult>`:
  - `NonExistent` — the target `quote2` call returned `exists=false`.
  - `RevertError(AbiDecodedErrorType)` — decoder recognized the revert (often a Rain error like `TokenSelfTrade`).
  - `RevertErrorDecodeFailed(AbiDecodeFailedErrors)` — revert present but not decodable by the registry.
  - `CorruptReturnData(String)` — malformed data when a multicall reports failure without decodable payload.

- `Error` represents top‑level failures of building or executing a batch:
  - URL parse errors, subgraph client errors, ABI decode errors when building targets, provider creation failures, and `MulticallError` when the overall transport fails.

- wasm conversions are implemented so both `FailedQuote` and `Error` convert to `JsValue` messages.


## RPC Layer: `rpc::batch_quote`

Purpose: execute `quote2` for many `QuoteTarget`s in one multicall.

Flow:
1. Parse RPC URLs into `Url` and build a `ReadProvider` via `mk_read_provider(&rpcs)` from `rain_orderbook_bindings`.
2. Create a dynamic `multicall` builder; override its address if `multicall_address` is provided.
3. Optionally pin to `block_number` via `BlockId::Number(block)`.
4. For each target, push `IOrderBookV5::quote2(QuoteV2)` into the multicall.
5. Await `aggregate3()`:
   - If the entire multicall returns `Err(MulticallError::CallFailed(bytes))`, decode the bytes via the Rain error selector registry and return a vector with the same per‑target error for each element (so callers still receive a `Vec<QuoteResult>` of the right length).
   - If other transport‑level errors occur, bubble them up as `Error::MulticallError` and do not return per‑target results.
6. For every `aggregate3` element:
   - `Ok(ret)` and `ret.exists == true` → `Ok(OrderQuoteValue::from(ret))`.
   - `Ok(ret)` and `ret.exists == false` → `Err(FailedQuote::NonExistent)`.
   - `Err(failure)` → decode `failure.return_data` via the registry into `FailedQuote::RevertError` or `FailedQuote::RevertErrorDecodeFailed`.

Notes:
- `gas` is currently accepted but unused (placeholder for future control).
- `block_number` lets callers obtain deterministic historical quotes.


## Subgraph Integration

- `OrderbookSubgraphClient` is used to fetch `orderBytes` either single (`order_detail`) or batch (`batch_order_detail`).
- `QuoteSpec` and `BatchQuoteSpec` use the helper `make_order_id(orderbook, order_hash)` to query the subgraph.
- ABI decoding of `orderBytes` constructs `OrderV4`, which is then embedded into `QuoteV2`.


## Order Pair Sweep: `order_quotes::get_order_quotes`

Purpose: given full `SgOrder` records (including token metadata), compute quotes for every valid input/output pair of the order and package human‑readable results.

Flow:
- Resolve `req_block_number`: use the provided `block_number` or fetch the current one via `ReadableClient::new_from_http_urls(rpcs.clone())?.get_block_number().await`.
- For each `SgOrder`:
  - Convert `order.order_bytes` into `OrderV4`.
  - For each combination of `validInputs × validOutputs` where `input.token != output.token`:
    - Build `pair_name` using token symbols from the subgraph record.
    - Build a `QuoteTarget` with `QuoteV2 { order, inputIOIndex, outputIOIndex, signedContext: [] }`.
  - Batch quote all constructed targets at `req_block_number`.
  - For each pair, push a `BatchOrderQuotesResponse` with either `data: Some(OrderQuoteValue)` and `success: true`, or `error: Some(<string>)` and `success: false`.

Result: a flat `Vec<BatchOrderQuotesResponse>` that UIs can render per pair.


## CLI: `cli` module and `src/main.rs`

Entrypoint:
- `src/main.rs` calls `rain_orderbook_quote::cli::main()` which initializes tracing, parses args, and delegates to `Quoter::run()`.

Arguments (selected):
- `--rpc <URL>` (required): JSON‑RPC endpoint.
- `--sg|--subgraph <URL>` (optional): subgraph endpoint; required when using specs.
- `--block-number <INTEGER>`: quote at a specific block.
- `--multicall-address <ADDRESS>`: override Multicall3 address.
- `--output <PATH>`: write JSON result to a file.
- `--no-stdout`: suppress stdout; useful with `--output`.
- `--pretty`: pretty‑print JSON.

Input formats (mutually exclusive via `Input` group):
- `-i|--input <HEX_STRING>`: Packed bytes representing one or more `QuoteSpec`s. Each spec is exactly 54 bytes: `[20 bytes orderbook][1 byte inputIO][1 byte outputIO][32 bytes order_hash]`. Length must be a non‑zero multiple of 54.
- `--target <ORDERBOOK_ADDRESS> <INPUT_IO_INDEX> <OUTPUT_IO_INDEX> <ORDER_BYTES>`: One or more fully specified targets (can repeat `--target`).
- `--spec <ORDERBOOK_ADDRESS> <INPUT_IO_INDEX> <OUTPUT_IO_INDEX> <ORDER_HASH>`: One or more specs (requires `--subgraph`).

Output format:
- Always JSON array (`QuoterResult`). For each element:
  - Success → `{ "maxOutput": "0x…", "ratio": "0x…" }`.
  - Failure → `{ "status": "error", "message": "…" }`.
- File output mirrors stdout; `--pretty` controls formatting.

Behavior:
- `--target …` → `BatchQuoteTarget.do_quote()` directly against RPC.
- `--spec … --subgraph …` → fetch order bytes, then batch quote.
- `--input … --subgraph …` → decode into specs, then same as above.


## Quote Debugger: `quote_debug` (native only)

Purpose: Help users debug a `quote2` call by forking a chain, executing the call, collecting interpreter traces, and decoding any revert reason.

- Types:
  - `NewQuoteDebugger { fork_url: Url, fork_block_number: Option<u64> }`.
  - `QuoteDebugger { forker: Forker }`.

- `QuoteDebugger::new(args)` constructs a forked EVM via `rain_interpreter_eval::Forker::new_with_fork`.
- `QuoteDebugger::debug(quote_target)`:
  - Validates IO indices with `QuoteTarget::validate()`.
  - Encodes `quote2` call and executes it against the forked EVM.
  - If reverted, optionally decode revert data using the Rain selector registry.
  - Returns `(RainEvalResult, Option<Result<AbiDecodedErrorType, AbiDecodeFailedErrors>>)` where the first element contains execution traces (stacks, etc.).


## Module Map

- `src/lib.rs`: module wiring and public re‑exports.
- `src/main.rs`: CLI binary entry.
- `src/error.rs`: `Error` and `FailedQuote` enums; wasm conversions.
- `src/rpc.rs`: batched multicall quoting and revert decoding.
- `src/quote.rs`: core quoting types (`OrderQuoteValue`, `QuoteTarget`, `BatchQuoteTarget`, `QuoteSpec`, `BatchQuoteSpec`) and subgraph integration.
- `src/order_quotes.rs`: utilities to compute quotes across all IO pairs for `SgOrder`s.
- `src/cli/input.rs`: input parsing for CLI; conversions from CLI args/hex to batch types.
- `src/cli/mod.rs`: CLI struct (`Quoter`), output wrapper types, `run()` logic, and `main()`.
- `src/quote_debug.rs`: fork‑based debugger returning traces and optional decoded revert information.


## External Dependencies of Note

- `alloy` primitives and `sol_types`: ABI encode/decode, EVM types, and contract call modeling.
- `rain_orderbook_bindings`: contract bindings (`IOrderBookV5`, provider helpers like `mk_read_provider`).
- `rain_orderbook_subgraph_client`: GraphQL client and types; provides `order_detail` and `batch_order_detail`.
- `rain-error-decoding`: decodes revert selectors (known/unknown) into structured errors.
- `alloy-ethers-typecast::ReadableClient`: convenience for fetching block number across multiple HTTP RPCs.
- `rain_math_float::Float`: canonical numeric type used in quotes; serializes to hex strings and provides safe equality/formatting.
- `wasm-bindgen-utils`: `Tsify` derives and helper macros (`impl_wasm_traits!`, `add_ts_content!`).


## Testing Overview

The crate has extensive unit tests across modules, including:
- CLI parsing and end‑to‑end runs with mocked RPC and subgraph servers.
- `rpc::batch_quote` success, transport errors, and both encoded and undecodable revert paths.
- `quote` module helpers (`get_id`, `validate`, subgraph fetches, batch alignment preserving optionals).
- `order_quotes::get_order_quotes` happy path and error propagation for malformed input or RPC URLs.
- Debugger tests that stand up a local anvil‑based EVM, deploy orders, and assert interpreter traces.

Tests use:
- `httpmock` for simulating JSON‑RPC and subgraph HTTP responses.
- `rain_orderbook_test_fixtures::LocalEvm` to spawn a local chain with tokens and an orderbook.


## Invariants and Edge Cases

- IO indices must be within bounds of the `OrderV4`’s `validInputs`/`validOutputs` (enforced by `QuoteTarget::validate()` and in debugger; RPC path assumes caller provides valid indices).
- Packed hex input for `-i|--input` must be a non‑zero multiple of 54 bytes; otherwise parsing fails.
- Multicall behavior:
  - Aggregate transport errors → top‑level `Error::MulticallError`.
  - Aggregate “call failed with bytes” → returns a per‑target `Err(FailedQuote::…)` vector to preserve shape.
- Missing subgraph orders:
  - Batch path returns `Err(FailedQuote::NonExistent)` in the corresponding positions.
- `gas` parameter currently reserved; intentionally unused.


## Typical Usage Patterns

- Library (native):
  - Build `QuoteTarget`s and call `BatchQuoteTarget::do_quote(rpcs, block, None, None)`.
  - Or build `QuoteSpec`s and call `BatchQuoteSpec::do_quote(subgraph_url, rpcs, block, None, None)`.

- CLI:
  - Direct targets: `cargo run -p rain_orderbook_quote -- --rpc <RPC> --target <ORDERBOOK> <IN_IO> <OUT_IO> <ORDER_BYTES>`
  - Specs via subgraph: `--spec <ORDERBOOK> <IN_IO> <OUT_IO> <ORDER_HASH> --sg <SUBGRAPH>`
  - Packed bytes: `-i <HEX> --sg <SUBGRAPH>` where `<HEX>` is repeated 54‑byte chunks.

- Debugging (native only):
  - Instantiate `QuoteDebugger` with a fork URL (e.g. local anvil or public RPC) and optional block.
  - Call `.debug(quote_target)` and inspect `RainEvalResult` traces and decoded revert, if any.


## Limitations and Future Work

- `gas` is not used yet; future revisions may allow customizing execution gas per call in multicall contexts.
- Subgraph fetch assumes ABI correctness in `orderBytes`; corrupted bytes cause `OrderDetailError`.
- RPC URL health is caller’s responsibility; the library supports multiple URLs and will error on unparseable inputs.

