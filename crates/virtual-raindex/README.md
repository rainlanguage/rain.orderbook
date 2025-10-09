# virtual-raindex

Virtual Raindex is a pure-Rust implementation of the Rain Orderbook. It keeps 
orders, vault balances, interpreter store values, and runtime metadata entirely
in memory while delegating Rainlang execution to an embedded interpreter host.
The crate powers quoting, taking, state management, and event replay
scenarios without depending on a live blockchain node, making it ideal for
simulation, backtesting, and deterministic ingestion workloads.

## Features

- **Deterministic virtual state** – mutate environment values, orders, vaults,
  store writes, and token decimals through a single `RaindexMutation` entry
  point.
- **Quote and take helpers** – run calculate-io and handle-io flows using the
  same interpreter context ordering as the on-chain orderbook.
- **Interpreter host abstraction** – plug in any host that implements
  `InterpreterHost`; a REVM-backed `RevmInterpreterHost` is provided out of the
  box.
- **Pluggable bytecode cache** – share interpreter and store bytecode across
  instances by supplying your own `CodeCache` implementation.
- **Snapshots for persistence** – capture and restore state with the `Snapshot`
  type for reproducible testing.
- **Event replay adapters** – translate on-chain OrderBook and Interpreter
  Store logs into `RaindexMutation`s for offline ingestion and parity checks.

## Crate Layout

The crate is organised by concern:

- `lib.rs` – re-exports the public API surface.
- `cache` – bytecode cache trait plus the `StaticCodeCache` reference
  implementation.
- `engine` – core execution flow including quoting, taking, and order
  post-task execution.
- `host` – interpreter host abstractions and the REVM-backed implementation.
- `state` – virtual state representation and mutation mechanics.
- `types` – request/response payloads shared by the engine.
- `events` – adapters that convert decoded logs into `RaindexMutation`s for
  OrderBook and Store contracts.
- `integration_tests.rs` – black-box tests covering quote/take flows plus
  end-to-end event replay parity against a local Anvil deployment.

## Getting Started

Add `virtual-raindex` to your workspace and instantiate a `VirtualRaindex`
engine with a bytecode cache and interpreter host:

```rust
use std::sync::Arc;

use alloy::primitives::Address;
use virtual_raindex::{
    host::RevmInterpreterHost,
    RaindexMutation,
    Result,
    StaticCodeCache,
    VirtualRaindex,
};

fn build_raindex(
    orderbook: Address,
) -> Result<VirtualRaindex<StaticCodeCache, RevmInterpreterHost<StaticCodeCache>>> {
    let cache = Arc::new(StaticCodeCache::default());
    let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
    let mut raindex = VirtualRaindex::new(orderbook, cache, host);

    raindex.apply_mutations(&[RaindexMutation::SetEnv {
        block_number: Some(1),
        timestamp: Some(1_700_000_000),
    }])?;

    Ok(raindex)
}
```

With an instance initialised, you can:

- call `raindex.quote(...)` to simulate calculate-io outputs;
- call `raindex.take_orders(...)` for a read-only take simulation;
- call `raindex.take_orders_and_apply_state(...)` to mutate state using the
  computed outcome;
- inspect `raindex.snapshot()` to capture the current state.
- feed decoded OrderBook / Store logs through `events::orderbook::orderbook_event_to_mutations`
  and `events::store::store_event_to_mutation` to rebuild state purely from
  on-chain events.

Remember to populate the bytecode cache with the interpreter and store
artifacts required by your orders before executing quotes or takes.

## Running Tests

The crate ships with integration coverage that exercises the quote and take
pipelines. Run the suite with:

```bash
cargo test -p virtual-raindex
```

If you are planning to embed the Virtual Raindex in another service, review the
integration tests for end-to-end usage patterns. In particular,
`event_ingestion_recreates_virtual_state` demonstrates replaying Anvil logs using
the event adapters and verifying that the reconstructed snapshot matches the
live virtual instance.
