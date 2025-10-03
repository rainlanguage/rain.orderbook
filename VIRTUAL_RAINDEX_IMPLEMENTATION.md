# Virtual Raindex Implementation Notes

## Snapshot Lifecycle
- Capture state via `VirtualRaindex::snapshot()` and rebuild with `VirtualRaindex::from_snapshot` when a ready cache/host is available.
- Use `virtual_raindex::snapshot::SnapshotBundle` to package snapshots for storage or transfer. Bundles include interpreter/store cache handles so callers can pre-load bytecode before hydration.
- `SnapshotBundle::from_snapshot` consumes a `Snapshot` and produces JSON-friendly records; `SnapshotBundle::into_snapshot` rebuilds the in-memory maps without requiring unsafe pointers.
- `VirtualRaindex::from_snapshot_bundle` validates that all interpreter/store addresses referenced by the bundle are present in the provided cache and returns the ready engine instance.

## Bytecode Cache Hydration
- `StaticCodeCache::from_artifacts` and `StaticCodeCache::with_pair` accept `(Address, &[u8])` tuples and validate that bytecode is non-empty before caching.
- Per-address insertions return `Result<()>`; reusing an address with different bytes now raises `RaindexError::BytecodeCollision { address, kind }` instead of silently overwriting.
- Empty or undecodable payloads raise `RaindexError::InvalidBytecodeEncoding { address, kind }`, allowing callers to surface data quality issues early.
- `StaticCodeCache::ingest_interpreters` / `ingest_stores` bulk-load artifacts from iterators and can be fed directly from DB blobs.

## Testing & Validation
- Standard host tests: `cargo test -p virtual-raindex` (or `nix develop -c cargo test -p virtual-raindex`).
- wasm host tests: `RUSTFLAGS="--cfg wasm_test" nix develop -c cargo test --target wasm32-unknown-unknown -p virtual-raindex --no-default-features --features web`.
- Snapshot round-trip expectations are covered in `crates/virtual-raindex/src/engine/tests.rs`; cache hydration guards and wasm serialization smoke tests live alongside them.

## Error Surface Changes
- `RaindexError` now includes `InvalidBytecodeEncoding` and `BytecodeCollision` variants in addition to `MissingBytecode`; update downstream handling accordingly.
- Snapshot helpers operate purely on owned data, ensuring wasm builds avoid raw pointer aliasing when serializing/deserializing state.
