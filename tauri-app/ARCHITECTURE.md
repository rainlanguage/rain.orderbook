# Tauri App — Architecture

This directory contains the Raindex desktop application built with Tauri (Rust + Svelte). It pairs a Svelte front end (Vite/SvelteKit client-only) with a Rust “backend” running in the same process that exposes native capabilities (filesystem, dialogs, Ledger, RPC) via Tauri `invoke` commands and emits UI events for toasts and transaction status.


## Overview

- Purpose
  - Native desktop app to compose, deploy, debug, and manage Rain Orderbook “algorithmic orders”.
  - Bridges UI workflows (Svelte) to workspace Rust crates for quoting, subgraph reads, calldata generation, and transaction execution.
- Targets
  - Cross‑platform Tauri bundle (macOS `.app/.dmg`, Windows `.msi/.nsis`, Linux `.deb`), using WebKit/Wry for the UI.
  - Runs fully offline for most flows; relies on configured RPCs/subgraphs for chain data.
- Upstream crates and libs (Rust)
  - `rain_orderbook_common`, `rain_orderbook_quote`, `rain_orderbook_subgraph_client`, `rain_orderbook_app_settings`, `rain_orderbook_bindings`, `rain-erc`, `rain-math-float`, Alloy, `alloy-ethers-typecast`.
- Upstream packages (TS)
  - `@rainlanguage/orderbook` (WASM SDK), `@rainlanguage/ui-components` (UI), WalletConnect, Ethers v5, CodeMirror (Rainlang), Sentry.

Typical development

```
# from repo root, inside a Nix shell
nix develop .#tauri-shell --command cargo tauri dev
```


## Runtime & State

- Front end (Svelte, client‑only)
  - `src/routes/+layout.ts` sets `export const ssr = false` and wires stores/providers in `+layout.svelte`.
  - Feature routes under `src/routes/*`: orders, vaults, settings, license and modals for quote/trade debug.
  - Sentry is initialized in `src/hooks.client.ts`/`src/lib/services/sentry.ts` (disabled by default via env; runtime‑toggleable).
- Native side (Rust/Tauri)
  - `src-tauri/src/main.rs` registers all `#[tauri::command]` handlers and manages shared state.
  - `SharedState` holds a `FuzzRunner` for charting/debug across invocations.
  - On Linux, sets `WEBKIT_DISABLE_COMPOSITING_MODE=1` to avoid a blank screen (known WebKitGTK issue).


## Commands (Rust side)

All commands are defined in `src-tauri/src/commands/*` and registered in `main.rs`. They are invoked from the UI via `@tauri-apps/api` `invoke()`.

- Chain (`commands/chain.rs`)
  - `get_chainid(rpcs)` — read chain ID via multi‑RPC client.
  - `get_block_number(rpcs)` — read latest block.
- Wallet (`commands/wallet.rs`)
  - `get_address_from_ledger(derivation_index, chain_id)` — query Ledger for the current address on a chain.
- Rainlang / DOTRAIN (`commands/dotrain.rs`, `commands/dotrain_add_order_lsp.rs`)
  - `parse_dotrain(rainlang, rpcs, block_number, deployer)` — fork‑parse Rainlang to bytecode.
  - Language services: `call_lsp_hover`, `call_lsp_completion`, `call_lsp_problems` for CodeMirror integration, optionally on a fork.
- Config & YAML (`commands/config.rs`)
  - `check_settings_errors(text[])`, `check_dotrain_with_settings_errors(dotrain, settings[])` — validate YAML with strict rules.
  - `get_deployments(dotrain, settings?)`, `get_scenarios(dotrain, settings?)` — parse and return typed maps for UI selection.
- Orders (`commands/order.rs`)
  - CSV/export: `orders_list_write_csv(path, subgraphArgs)`.
  - Mutations: `order_add(dotrain, deployment, transactionArgs)`, `order_remove(chain_id, id, transactionArgs, subgraphArgs)`.
  - Calldata helpers: `order_add_calldata(...)`, `order_remove_calldata(id, subgraphArgs)`.
  - Composition & validation: `compose_from_scenario(dotrain, settings?, scenario)`, `validate_spec_version(dotrain, settings[])`.
- Vaults (`commands/vault.rs`)
  - CSV/export: `vaults_list_write_csv(path, subgraphArgs)`, `vault_balance_changes_list_write_csv(id, path, subgraphArgs)`.
  - Mutations: `vault_deposit(depositArgs, transactionArgs)`, `vault_withdraw(withdrawArgs, transactionArgs)`.
  - Calldata helpers: `vault_deposit_approve_calldata(...)`, `vault_deposit_calldata(...)`, `vault_withdraw_calldata(...)`.
- Debug & Analysis
  - Quotes (`commands/order_quote.rs`): `debug_order_quote(...)` to simulate quotes for an order.
  - Trades (`commands/trade_debug.rs`): `debug_trade(tx_hash, rpcs)` to replay a transaction and return evaluation results.
  - Charts (`commands/charts.rs`): `make_charts(dotrain, settings?)` and `make_deployment_debug(dotrain, settings, block_numbers?)` via the fuzz runner.
- App (`commands/app.rs`)
  - `get_app_commit_sha()` — expose embedded commit SHA for diagnostics.

Errors are unified via `src-tauri/src/error.rs` as `CommandError/CommandResult`, translating many upstream error types to user‑readable strings for the UI.


## IPC Events (Rust → UI)

- Toasts (`src-tauri/src/toast.rs`)
  - Emits `toast` with `{ message_type, text }` for UI notifications. Consumed by `src/lib/stores/toasts.ts`.
- Transaction status (`src-tauri/src/transaction_status.rs`)
  - `TransactionStatusNoticeRwLock` tracks a tx’s lifecycle (`Initialized`, `PendingPrepare`, `PendingSign`, `Sending`, `Confirmed`, `Failed`).
  - Emits `transaction_status_notice` events, consumed by `src/lib/stores/transactionStatusNotice.ts` to display transient banners.
  - Types are shared to TS via wasm‑generated bindings (see below).


## UI & Services (Svelte)

- Services layer (`src/lib/services/*`)
  - Thin wrappers around `invoke()` for each command: `order.ts`, `vault.ts`, `chain.ts`, `config.ts`, `chart.ts`, `authoringMeta.ts`, `wallet.ts`, `app.ts`.
  - Execution helpers: `executeLedgerOrder.ts` (native flow), `executeWalletConnectOrder.ts` (WalletConnect + Ethers), `ethersTx.ts`.
  - Language services bridge: `langServices.ts` wires CodeMirror Rainlang hover/completion/problems to Tauri LSP commands and a fork block number store.
  - Settings application: `applySettings.ts`, diagnostics mapping in `configCodemirrorProblems.ts`.
- Stores (`src/lib/stores/*`)
  - WalletConnect orchestration (`walletconnect.ts`), selected chains/accounts/filters (`settings.ts`), toasts, tx notices, dark mode, etc.
  - `walletconnect.ts` discovers working RPCs per chain before connecting, and tracks the active chain/account.
- Components & routes
  - Orders: list, detail, remove; vaults: list, deposit/withdraw; add‑order screen with CodeMirror + charts; debug modals for quotes/trades.
  - Many UI pieces come from `@rainlanguage/ui-components`.


## Bindings generation (TS types for events)

- The Rust crate exposes a tiny `src-tauri/src/types/*` surface that is compiled for `wasm32-unknown-unknown` to derive TS types (`wasm_bindgen_utils` + `tsify`).
- Build pipeline
  - `npm run build-wasm` runs `cargo build --target wasm32-unknown-unknown --lib -r` in `src-tauri/`.
  - `scripts/build.js` runs `wasm-bindgen` and writes `src/lib/types/tauriBindings.ts` with the event payload typings:
    - `ToastMessageType`, `ToastPayload`, `TransactionStatus`, `TransactionStatusNotice`.
  - UI imports these types for strict event payloads.


## Build, Dev, Test

- Dev
  - `nix develop .#tauri-shell --command cargo tauri dev` (from repo root). Tauri runs the Svelte dev server (`vite dev`) per `tauri.conf.json#build.beforeDevCommand`.
- Build
  - `nix develop .#tauri-shell --command cargo tauri build` to produce installers/bundles.
  - Frontend build is configured via `vite.config.ts`; source maps can be uploaded to Sentry in release mode.
- Frontend scripts (from `tauri-app/`)
  - `npm run dev`, `npm run build`, `npm run test` (Vitest, jsdom), `npm run build-bindings` to regenerate TS event types.


## Configuration

Copy `.env.example` to `.env` in this directory and set as needed:

- `VITE_WALLETCONNECT_PROJECT_ID` — required for WalletConnect.
- Sentry (optional): `VITE_SENTRY_DSN`, `VITE_SENTRY_RELEASE`, `VITE_SENTRY_ENVIRONMENT`, `VITE_SENTRY_FORCE_DISABLED`.
- Build‑time source maps upload (optional release flow): `SENTRY_SOURCE_MAPS_ENABLED`, `SENTRY_ORG`, `SENTRY_PROJECT`, `SENTRY_AUTH_TOKEN`.

Tauri allowlist and bundling are configured in `src-tauri/tauri.conf.json`:

- Allowlist: `shell.open`, `dialog.open/save`, `fs.readFile/writeFile`, `window.startDragging`, and `os.*`.
- Bundle targets: `deb`, `nsis`, `msi`, `app`, `dmg`, `updater`.
- macOS frameworks include USB libraries required for Ledger.


## Directory Layout

- `src/` — Svelte app
  - `routes/` — pages: home, orders, vaults, settings, license.
  - `lib/services/` — invoke clients, execution helpers, charts, language services, sentry.
  - `lib/stores/` — app state (walletconnect, settings, toasts, tx notices, theme).
  - `lib/components/` — app‑specific wrappers and modals.
  - `lib/types/tauriBindings.ts` — auto‑generated TS types from Rust.
- `src-tauri/` — Rust side
  - `src/commands/` — Tauri commands grouped by domain (`order`, `vault`, `chain`, `config`, `charts`, `wallet`, `dotrain*`, `trade_debug`, `order_quote`).
  - `src/{error,toast,transaction_status,shared_state}.rs` — error unification, UI event emitters, tx status tracker, shared FuzzRunner state.
  - `tauri.conf.json` — allowlist, bundling, window config, pre‑dev/build hooks.
  - `Cargo.toml` — depends on workspace crates via `path` and Alloy; builds as `cdylib` for bindings.
- `scripts/build.js` — wasm‑bindgen step to emit TS typings for event payloads.


## How It Fits The Workspace

- This app is a consumer of the Rust crates under `crates/*` and the UI kit under `packages/*`.
- `src-tauri/` is not part of the Cargo workspace but depends on those crates by path.
- If you add features to core crates (e.g., new order/vault operations or analysis), expose them via a new Tauri command and a matching UI service wrapper.


## References

- Crates: `crates/ARCHITECTURE.md`, `crates/common/ARCHITECTURE.md`, `crates/quote/ARCHITECTURE.md`, `crates/settings/ARCHITECTURE.md`.
- JS packages: `packages/orderbook/ARCHITECTURE.md`, `packages/webapp/ARCHITECTURE.md`, `packages/ui-components/ARCHITECTURE.md`.
- Dev commands & repo tips: see repository README and the project “Repository Guidelines”.

