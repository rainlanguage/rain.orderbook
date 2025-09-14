# Repository Guidelines

## Project Structure & Module Organization
- Solidity contracts: `src/`, tests in `test/` with fixtures in `test-resources/`.
- Rust workspace: `crates/*` (e.g., `cli`, `common`, `bindings`, `js_api`, `quote`, `subgraph`, `settings`, `math`, `integration_tests`).
- JavaScript/Svelte: `packages/*` — `webapp`, `ui-components`, `orderbook` (wasm wrapper published to npm).
- Desktop app: `tauri-app` (Rust + Svelte; `src-tauri` excluded from Cargo workspace).
- Subgraph and tooling: `subgraph/`, `script/`, helper scripts like `prep-all.sh`.

## Build, Test, and Development Commands
- Run everything inside a Nix shell. Either enter `nix develop` first and run commands normally, or prefix with `nix develop -c <cmd>`. Use shell attrs when needed (e.g., `nix develop .#tauri-shell`).
- Bootstrap: `./prep-all.sh` (installs deps and builds workspaces).
- Rust: `nix develop -c cargo build --workspace`; tests: `nix develop -c cargo test`.
- Solidity (Foundry): `nix develop -c forge build`; tests: `nix develop -c forge test`.
- Webapp: `cd packages/webapp && nix develop -c npm run dev`.
- Tauri: `nix develop .#tauri-shell --command cargo tauri dev`.
- JS workspaces (top-level): `npm run test`, `npm run build:ui`, `npm run build:orderbook`.
- WASM bundle: `cd packages/orderbook && npm run build-wasm`.

## Coding Style & Naming Conventions
- Rust: format with `cargo fmt --all`; lint with `cargo clippy --all-targets -- -D warnings`. Crates/modules use `snake_case`; types `PascalCase`.
- TS/Svelte: `npm run format`, `npm run lint`, `npm run check` in each package. Components `PascalCase.svelte`; files otherwise kebab/snake as appropriate.
- Solidity: `forge fmt`; compiler `solc 0.8.25` (see `foundry.toml`).

## Testing Guidelines
- Rust: `cargo test`; integration tests live in `crates/integration_tests`. Prefer `insta` snapshots and `proptest` where helpful.
- TS/Svelte: `npm run test` (Vitest). Name files `*.test.ts`/`*.spec.ts`.
- Solidity: `forge test` (add fuzz/property tests where relevant).

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`.
- PRs must: describe scope/approach, link issues, include screenshots/GIFs for UI changes, update/ add tests, and pass CI.
- Quick preflight: `nix develop -c npm run lint-format-check:all && nix develop -c cargo clippy -- -D warnings`.

## Security & Configuration Tips
- Never commit secrets. Copy `.env.example` files (root, `packages/webapp`, `tauri-app`) and populate `PUBLIC_WALLETCONNECT_PROJECT_ID` / `VITE_WALLETCONNECT_PROJECT_ID` as required.

## Agent-Specific Instructions
- Prefer syntax-aware search with ast-grep: Rust `sg --lang rust -p '<pattern>'`; TS `sg --lang ts -p '<pattern>'`. Use Nix shells for tool parity.
- Architecture context: when working in any directory, check for an `ARCHITECTURE.md` file in the current working directory and read it first to understand local architecture before making changes.
