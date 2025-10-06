**Overview**
- Purpose: `rain_orderbook_js_api` exposes a single, browser-friendly WebAssembly surface for the Rain Orderbook application. It bridges YAML-based “dotrain” order configuration, on-chain ERC‑20/token metadata, and contract call generation into a typed JavaScript/TypeScript API.
- Target: Compiles as a `cdylib` for wasm and is designed to be consumed from JS environments (webapps, Tauri). All public APIs are exported via `wasm_bindgen_utils` macros and return ergonomic results with rich, user‑readable errors.
- Scope: Includes high-level GUI helpers for interactive order building, a fetchable registry of orders, and low-level helpers for hashing and ABI calldata generation. It re-exports certain sibling crates so their wasm bindings are reachable from a single import.

**Build & Targets**
- Crate type: `cdylib` (WASM output). Most modules are `#[cfg(target_family = "wasm")]` as they are JS/GUI facing.
- Key dependencies: `wasm-bindgen-utils`, `alloy` (ABI/primitives), `rain_orderbook_*` crates for app models + on-chain helpers, `tokio` (async), `reqwest` (HTTP, registry), `flate2`/`base64`/`bincode`/`sha2` (state serialization), `strict-yaml-rust` (YAML AST).
- TypeScript support: Adds TS definitions for `Address` and `Hex` template literal types and uses `tsify` to describe return/param types of exported structs.

**Top-Level Layout**
- `src/lib.rs`
  - Exposes modules only when targeting wasm: `bindings`, `gui`, `registry`, `yaml`.
  - Re-exports crates so their wasm bindings are available from this single module: `rain_orderbook_app_settings`, `rain_orderbook_common`, `rain_orderbook_subgraph_client`.
  - Appends a small TS section defining `Address` and `Hex` template literal types for better typing on the JS side.

**FFI & Error Conventions**
- Functions and impl blocks use `#[wasm_export]` (from `wasm_bindgen_utils`), which:
  - Exports JS-callable functions and classes with Promise-based async where needed.
  - Bridges `Result<T, E>` into JS objects with `.value` on success or an `.error` containing a serialized `WasmEncodedError` with both `msg` and `readable_msg`.
  - Uses hints like `unchecked_return_type` and `preserve_js_class` to fine-tune TS output.
- Data structs use `#[derive(Tsify)]` to generate accurate TS types (e.g., `Hex`, `Map<…>`, arrays), and many address-like types are annotated as TS `string` for interop.

**Modules**

- `bindings` (src/bindings/mod.rs)
  - Purpose: Low-level helpers exposed to JS for hashing and ABI encoding independent of the GUI flow.
  - Key types and exports:
    - `TakeOrdersCalldata(Bytes)` as an opaque JS type for encoded calldata.
    - `getOrderHash(order: OrderV4) -> string`: ABI-encodes `OrderV4` and returns `keccak256` with `0x` prefix.
    - `getTakeOrders3Calldata(config: TakeOrdersConfigV4) -> TakeOrdersCalldata`: ABI-encodes a `takeOrders3` call for the on-chain OrderBook.
    - `keccak256(bytes: Uint8Array) -> string` and `keccak256HexString(hex: string) -> string`.
  - Errors: `Error::FromHexError` mapped to JS with human-readable message.

- `gui` (src/gui/…)
  - Purpose: High-level, stateful orchestrator for interactive order creation from a dotrain (YAML + Rainlang) configuration. Encapsulates reading config, managing user inputs, querying token metadata, validating fields, and generating contract call data for deployment.
  - Core type: `DotrainOrderGui`
    - Fields: `dotrain_order` (parsed configuration), `selected_deployment`, `field_values` and `deposits` (with preset tracking), and an optional `state_update_callback` JS function.
    - Construction:
      - `DotrainOrderGui.getDeploymentKeys(dotrain: string) -> string[]` parses `gui.deployments`.
      - `DotrainOrderGui.newWithDeployment(dotrain, selectedDeployment, stateUpdateCallback?) -> DotrainOrderGui` validates the deployment and bootstraps a GUI instance.
    - Config accessors:
      - `getGuiConfig() -> GuiCfg`, `getCurrentDeployment() -> GuiDeploymentCfg` (filtered for the active deployment).
      - `getOrderDetails(dotrain) -> NameAndDescriptionCfg` (static), `getDeploymentDetails(dotrain) -> Map<string, NameAndDescriptionCfg>`, `getDeploymentDetail(dotrain, key) -> NameAndDescriptionCfg`.
      - `getCurrentDeploymentDetails() -> NameAndDescriptionCfg`.
    - Token metadata:
      - `getTokenInfo(key) -> TokenInfo`: returns address/decimals/name/symbol. Falls back to on-chain queries if YAML is incomplete.
      - `getAllTokenInfos() -> TokenInfo[]`: collects token keys from `select-tokens` or order IO, then fetches details as needed.
    - Dotrain/Rainlang exports:
      - `generateDotrainText() -> string`: emits full dotrain text (YAML frontmatter + `---` + Rainlang body), preserving the current config.
      - `getComposedRainlang() -> string`: updates scenario bindings from saved field values and composes Rainlang ready for preview.

  - Submodules
    - `field_values.rs`
      - User-controlled inputs declared under `gui.deployments[*].fields`.
      - Setters and getters:
        - `setFieldValue(binding, value)`: validates (if rules exist), detects preset matches, stores as either preset index or custom value, and triggers the state callback.
        - `setFieldValues([{field, value}, …])`: batch equivalent.
        - `unsetFieldValue(binding)`.
        - `getFieldValue(binding) -> { field, value, isPreset }` expands presets to actual values for display.
        - `getAllFieldValues() -> FieldValue[]`.
        - `getFieldDefinition(binding) -> GuiFieldDefinitionCfg` and `getAllFieldDefinitions(filterDefaults?)` (filter by has default/no default), `getMissingFieldValues()`.
      - Validation: delegated to `validation.rs` using YAML-provided rules (Number min/max/exclusive bounds, String min/max length, Boolean exact `"true"|"false"`). Uses `rain_math_float::Float` for precise numeric comparisons.

    - `deposits.rs`
      - User deposit amounts declared under `gui.deployments[*].deposits`.
      - Helpers:
        - `getDeposits() -> TokenDeposit[]` expanding presets to actual values and pairing with token addresses.
        - `setDeposit(tokenKey, amount)` validates per-token rules (min/max/exclusive), detects presets, stores, and triggers state callback.
        - `unsetDeposit(tokenKey)`, `getDepositPresets(tokenKey) -> string[]`, `getMissingDeposits() -> string[]`, `hasAnyDeposit() -> boolean`.
        - `check_deposits()` (internal) enforces that all required deposits are set for the current deployment.

    - `select_tokens.rs`
      - For deployments that declare `select-tokens`, users supply token contracts at runtime.
      - Features:
        - `getSelectTokens() -> GuiSelectTokensCfg[]` and `checkSelectTokens()`.
        - `isSelectTokenSet(key) -> boolean`.
        - `setSelectToken(key, address)` fetches ERC‑20 metadata via RPC (derived from the deployment’s network) and writes token records back into the dotrain YAML; triggers state callback.
        - `unsetSelectToken(key)` removes previously selected token records.
        - `areAllTokensSelected() -> boolean`.
        - Token discovery: `getAllTokens(search?) -> TokenInfo[]` returns all tokens for the active network. If metadata is missing in YAML, it fetches on-chain, dedupes by address, and optionally filters by name/symbol/address substring. Concurrency is limited by `MAX_CONCURRENT_FETCHES`.
        - `getAccountBalance(tokenAddress, owner) -> AccountBalance` reads ERC‑20 decimals and balance and returns both raw and formatted balance.

    - `order_operations.rs`
      - Generates all calldata required to deploy orders and related flows.
      - Internal preparation:
        - `prepare_calldata_generation` validates select-tokens, ensures field values exist as needed, populates vault IDs, and updates scenario bindings before generating any calldata.
        - `get_orderbook()` and `get_transaction_args()` collect the orderbook address and RPCs for downstream calls.
        - `get_deposits_as_map()` and `get_vaults_and_deposits()` resolve deposit amounts by token/address and match them to order outputs + vaults.
      - Allowance/approvals:
        - `checkAllowances(owner) -> AllowancesResult`: queries current allowances for each deposit token against the orderbook.
        - `generateApprovalCalldatas(owner) -> ApprovalCalldataResult`: compares allowances to required deposit amounts; emits ERC‑20 `approve` calldatas only when needed.
      - Deposits:
        - `generateDepositCalldatas() -> DepositCalldataResult`: builds `deposit3` calldatas for non-zero deposits using vault IDs (fetches decimals on-chain if missing in YAML).
      - Add order:
        - `generateAddOrderCalldata() -> AddOrderCalldataResult`: composes Rainlang, builds an `AddOrderArgs` from the deployment, and returns the ABI-encoded call.
      - Combined deployment:
        - `generateDepositAndAddOrderCalldatas() -> DepositAndAddOrderCalldataResult`: constructs a `multicall` that first performs `addOrder`, then all deposits.
        - `getDeploymentTransactionArgs(owner) -> DeploymentTransactionArgs`: packages approval calldatas (with token symbol for UX), multicall calldata, orderbook address, and chain ID for a one-shot deployment flow.
      - Vault IDs:
        - `setVaultId(type: 'input'|'output', tokenKey, vaultId?: string)`, `getVaultIds() -> IOVaultIds`, and `hasAnyVaultId() -> boolean`.
      - Types exposed for JS: `AllowancesResult`, `ApprovalCalldataResult|DepositCalldataResult|AddOrderCalldataResult|DepositAndAddOrderCalldataResult`, `ExtendedApprovalCalldata`, `DeploymentTransactionArgs`, `IOVaultIds`. A `WithdrawCalldataResult` type exists but no public generator yet.

    - `state_management.rs`
      - End-to-end state persistence and restoration:
        - `serializeState() -> string`: bincode-serializes a compact state (field values and deposit presets, selected tokens, vault IDs, selected deployment) then gzips and base64-encodes. Also embeds a SHA‑256 of the full dotrain to prevent mismatched restores.
        - `DotrainOrderGui.newFromState(dotrain, serialized, callback?) -> DotrainOrderGui`: validates the hash against the provided dotrain, rebuilds internal maps, replays selected tokens and vault IDs back into the YAML/documents, and returns a fully restored instance.
        - `executeStateUpdateCallback()`: manually triggers the callback by passing the latest `serializeState()` string. Most mutating methods call this automatically.
        - `getAllGuiConfig() -> AllGuiConfig`: returns all front-end relevant config slices grouped for progressive UI building (fields by required/optional, deposits, order inputs/outputs).

    - `validation.rs`
      - Uniform validation library used by `field_values` and `deposits`:
        - Numbers: `minimum`, `exclusive-minimum`, `maximum`, `exclusive-maximum`; rejects negatives; precise decimal support via `Float`.
        - Strings: `min-length`, `max-length` (length measured on trimmed strings).
        - Booleans: accepts only `"true"` or `"false"`.
      - Errors (`GuiValidationError`) carry contextual, user-readable messages; surfaced to JS via `GuiError::ValidationError`.

  - Error type for the GUI: `GuiError`
    - Captures configuration, selection, validation, I/O, chain, and serialization errors.
    - Provides `to_readable_msg()` with end-user friendly explanations.
    - Implements conversions to `JsValue` and `WasmEncodedError` for FFI.

- `registry` (src/registry.rs)
  - Purpose: Fetches a remote registry file that lists one shared settings YAML followed by one or more `.rain` order files. Produces merged dotrain content per order and can directly construct a `DotrainOrderGui` instance.
  - Registry format:
    - First non-empty line: settings YAML URL (no key)
    - Subsequent lines: `"<orderKey> <url-to-order.rain>"`
  - Flow:
    - `DotrainRegistry.new(registryUrl)` → fetch registry text → parse → fetch settings → fetch all orders (concurrently) → store in-memory.
    - `getAllOrderDetails()` → parse order-level metadata for every merged dotrain.
    - `getOrderKeys()` → keys from `order_urls`.
    - `getDeploymentDetails(orderKey)` → deployment name/description map for a specific order.
    - `getGui(orderKey, deploymentKey, stateCallback?)` → merge `settings + order` and produce a `DotrainOrderGui` instance.
  - Errors: `DotrainRegistryError` covers fetch/parse/HTTP/URL issues and wraps `GuiError`. Also returns human-readable messages.

- `yaml` (src/yaml/mod.rs)
  - Purpose: Wasm-friendly wrapper around orderbook YAML parsing to retrieve an `OrderbookCfg` by contract address.
  - Exports:
    - `OrderbookYaml.new([yamlSources], validate?) -> OrderbookYaml`: parse/merge/optionally validate sources.
    - `OrderbookYaml.getOrderbookByAddress(address) -> OrderbookCfg`.
  - Errors: `OrderbookYamlError` with readable messaging, converted to JS.

**External Crates & Interactions**
- `rain_orderbook_app_settings`: typed config model + YAML parsing helpers for GUI sections, deployments, networks, orders, select-tokens, and validation rules.
- `rain_orderbook_common`: higher-level order manipulation (compose Rainlang, add order args), ERC‑20 RPC client, transaction helpers, and formatting utilities.
- `rain_orderbook_bindings`: generated Solidity bindings for `IOrderBookV5` (e.g., `deposit3`, `multicall`, `takeOrders3`).
- `alloy`: ABI encoding/decoding, primitives (`Address`, `Bytes`, `U256`, keccak256), and Solidity type utilities.
- `wasm-bindgen-utils`: export macro, JS bridging, `WasmEncodedError` packaging.

**Data Flow & Typical Lifecycle**
- From dotrain → GUI → calldata:
  - Parse dotrain (frontmatter YAML + Rainlang body) with `DotrainOrder::create`.
  - Initialize GUI with a deployment key.
  - Optional: select tokens via on-chain metadata, set field values (with validation), set deposit amounts (with validation), and set vault IDs.
  - Generate approvals if needed, deposits, add order calldata, or a combined multicall. Transaction args include orderbook address and chain ID.
- State persistence:
  - Any setter triggers `executeStateUpdateCallback()` with a gzipped/base64 state snapshot that includes a dotrain content hash. `newFromState` restores and protects against mismatched content.
- Token metadata:
  - Prefer YAML cache when available; otherwise, query chain via current network’s RPC(s). Concurrency for token info lookups is capped.

**TypeScript Surface**
- Most exported structs are `Tsify`’d, and methods use `unchecked_return_type` for readable TS types:
  - Example: `getVaultIds()` returns a `Map<string, Map<string, string | undefined>>` keyed by `"input"`/`"output"` and token keys.
  - Calldata types are exposed as `Hex` or `Hex[]`, addresses as `string` with TS template literal types appended by `lib.rs`.

**Testing Notes**
- Uses `wasm-bindgen-test` to exercise behavior within wasm targets.
- Many tests validate:
  - Validation errors and their readable messages.
  - Deposit/field setters, preset detection, and getters.
  - Select-token flows and token discovery, including search and dedupe.
  - Vault ID setting and query helpers.
  - State serialization and restoration roundtrips (including hash mismatch protection).
  - Registry parsing/fetching logic; non-wasm tests use `httpmock` to simulate HTTP servers.

**Edge Cases & Notes**
- If YAML is missing token metadata, the crate queries the chain; callers should expect async RPC usage and potential network failures in those code paths.
- `WithdrawCalldataResult` exists as a type placeholder; no public generator currently uses it.
- Many GUI methods error if `select-tokens` is configured but tokens are not yet selected, or if required field values/deposits are missing. These error cases surface clear `readable_msg`s.
- When decimals are absent in YAML, they are fetched on demand before encoding deposits.

**How To Use (High-Level)**
- Single order flow:
  - `const gui = await DotrainOrderGui.newWithDeployment(dotrain, deploymentKey, onStateChanged?)`
  - Fill inputs: `setFieldValue`, `setDeposit`, optionally `setSelectToken`, `setVaultId`.
  - Generate data: `generateAddOrderCalldata` or `generateDepositAndAddOrderCalldatas`; or get the full package from `getDeploymentTransactionArgs(owner)`.
  - Persist UI state: read `serializeState()`; restore later with `DotrainOrderGui.newFromState(dotrain, serialized, callback?)`.
- Multiple orders via registry:
  - `const registry = await DotrainRegistry.new(registryUrl)` → inspect orders/deployments → `await registry.getGui(orderKey, deploymentKey, onStateChanged?)`.

**Summary**
- `rain_orderbook_js_api` is the JS/WASM gateway for building, validating, and deploying Rain Orderbook orders from YAML+Rainlang definitions. It centralizes: YAML parsing and validation, user input state, token selection and metadata, field and deposit validation, vault ID management, transaction calldata generation (approvals, deposits, add order, multicall), registry-driven content fetching, and robust error handling—exposed as a typed, ergonomic TypeScript surface.

