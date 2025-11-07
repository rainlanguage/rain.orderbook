# Raindex SDK

A TypeScript/JavaScript SDK for interacting with Raindex orderbook contracts, providing comprehensive functionality for order management, configuration parsing, and blockchain interactions.

## What is Raindex?

Raindex is an **onchain orderbook contract** that enables users to deploy complex, perpetual trading algorithms using **Rainlang**, a domain-specific language interpreted onchain. Learn more about Rainlang in the [official documentation](https://docs.rainlang.xyz/intro).

### How It Works

- **Dynamic Orders**: Unlike traditional orderbooks, Raindex orders contain algorithms that determine token movements based on real-time conditions
- **Vault System**: Users deposit tokens into vaults (virtual accounts) instead of using token approvals
- **Multi-Token Strategies**: Orders can reference multiple input/output vaults for sophisticated trading scenarios
- **Perpetual Execution**: Strategies remain active until explicitly removed by the owner
- **Decentralized Execution**: Third-party fillers execute trades by capitalizing on arbitrage opportunities

## SDK Overview

This SDK provides Rust-powered WebAssembly bindings for orderbook functionality, enabling developers to:

- **Query Orders & Trades**: Search orders across multiple networks, fetch order details, and track trade history
- **Execute Quotes**: Get real-time quotes for trading pairs with maximum output amounts and IO ratios
- **Manage Vaults**: Query vault balances, generate deposit/withdraw calldata, and track vault activity
- **Parse Configurations**: Validate YAML files defining networks, tokens, orderbooks, and subgraph endpoints
- **Generate Transactions**: Create ABI-encoded calldata for adding/removing orders and vault operations
- **Track Performance**: Monitor order volume, vault balance changes, and trading metrics over time

## Prerequisites

Before using this SDK, ensure you have:

- Node.js >= 22
- A Web3 provider (e.g., ethers.js, viem)
- A YAML configuration file (see [example configuration](https://github.com/rainlanguage/rain.strategies/blob/main/settings.yaml))

## Installation

```bash
npm install @rainlanguage/orderbook
```

## Quick Start

All SDK calls return `WasmEncodedResult<T>`; always check `.error` before touching `.value`.

### 1. Load orderbook settings & create a client

```ts
import fs from 'node:fs/promises';
import { OrderbookYaml, RaindexClient } from '@rainlanguage/orderbook';

const yamlSources = [await fs.readFile('./settings.yaml', 'utf8')];

const orderbookResult = OrderbookYaml.new(yamlSources, true);
if (orderbookResult.error) throw new Error(orderbookResult.error.readableMsg);
const orderbookYaml = orderbookResult.value;

const clientResult = RaindexClient.new(yamlSources);
if (clientResult.error) throw new Error(clientResult.error.readableMsg);
const client = clientResult.value;
```

Pass `true` as the second argument to `OrderbookYaml.new` / `RaindexClient.new` when you want strict schema validation.

### 2. Query orders with filters & pagination

```ts
import type { ChainIds, GetOrdersFilters } from '@rainlanguage/orderbook';

const chainIds: ChainIds = [1, 137, 8453];
const filters: GetOrdersFilters = {
  owners: ['0x1234...'],
  active: true,
  tokens: ['0xTokenAddress']
};

const ordersResult = await client.getOrders(chainIds, filters, 1);
if (ordersResult.error) throw new Error(ordersResult.error.readableMsg);
const orders = ordersResult.value; // RaindexOrder[]

const first = orders[0];
const vaultList = first.vaultsList; // RaindexVaultsList helper
const rawVaults = vaultList.items; // RaindexVault[]

const tradesResult = await first.getTradesList();
if (tradesResult.error) throw new Error(tradesResult.error.readableMsg);

const tradeDetailResult = await first.getTradeDetail('0xTradeId');
if (tradeDetailResult.error) throw new Error(tradeDetailResult.error.readableMsg);

const quotesResult = await first.getQuotes();
if (quotesResult.error) throw new Error(quotesResult.error.readableMsg);
```

Additional helpers worth wiring up:

- `client.getOrderByHash(chainId, orderbookAddress, orderHash)` – fetch a single order with full vault metadata.
- `client.getAddOrdersForTransaction(...)` / `client.getRemoveOrdersForTransaction(...)` – diff deployments and removals by transaction hash.
- `client.getTransaction(orderbookAddress, txHash)` – inspect who sent a transaction, the block number, and timestamp.

### 3. Work with vaults & Floats

```ts
import { Float, type GetVaultsFilters } from '@rainlanguage/orderbook';

const vaultFilters: GetVaultsFilters = {
  owners: ['0x1234...'],
  hideZeroBalance: true
};

const vaultsResult = await client.getVaults([14], vaultFilters, 1);
if (vaultsResult.error) throw new Error(vaultsResult.error.readableMsg);
const vaultsList = vaultsResult.value; // RaindexVaultsList

const withdrawableResult = vaultsList.getWithdrawableVaults();
if (withdrawableResult.error) throw new Error(withdrawableResult.error.readableMsg);
const withdrawableVaults = withdrawableResult.value;

const vault = withdrawableVaults[0];
const historyResult = await vault.getBalanceChanges();
if (historyResult.error) throw new Error(historyResult.error.readableMsg);

const depositAmount = Float.parse('10.5');
if (depositAmount.error) throw new Error(depositAmount.error.readableMsg);
const depositCalldataResult = await vault.getDepositCalldata(depositAmount.value);
if (depositCalldataResult.error) throw new Error(depositCalldataResult.error.readableMsg);

const withdrawAmount = Float.parse('2');
if (withdrawAmount.error) throw new Error(withdrawAmount.error.readableMsg);
const withdrawCalldataResult = await vault.getWithdrawCalldata(withdrawAmount.value);
if (withdrawCalldataResult.error) throw new Error(withdrawCalldataResult.error.readableMsg);

const approvalResult = await vault.getApprovalCalldata(depositAmount.value);
if (approvalResult.error) throw new Error(approvalResult.error.readableMsg);

const allowanceResult = await vault.getAllowance();
if (allowanceResult.error) throw new Error(allowanceResult.error.readableMsg);
const allowance = allowanceResult.value;
```

`RaindexVaultsList` also exposes `getWithdrawCalldata()` (builds a multicall to empty every vault with a balance), `pickByIds([...])`, and `concat(otherList)` if you need to restructure vault groups before submitting a transaction.

### 4. Generate quotes & calldata

```ts
import { getOrderHash, getTakeOrders3Calldata } from '@rainlanguage/orderbook';

const orderHashResult = getOrderHash(orderV4Struct);
if (orderHashResult.error) throw new Error(orderHashResult.error.readableMsg);

const takeOrdersResult = getTakeOrders3Calldata(takeOrdersConfig);
if (takeOrdersResult.error) throw new Error(takeOrdersResult.error.readableMsg);
const takeOrdersCalldata = takeOrdersResult.value; // hex string ready for the contract

const removeCalldataResult = await first.getRemoveCalldata();
if (removeCalldataResult.error) throw new Error(removeCalldataResult.error.readableMsg);

const quotesResult = await first.getQuotes();
if (quotesResult.error) throw new Error(quotesResult.error.readableMsg);
```

Every `RaindexOrder` exposes `vaultsList`, `inputsList`, `outputsList`, and `inputsOutputsList`, so you can quickly scope which vault IDs map to which IO leg before building calldata.

### 5. Optional: sync a local cache

```ts
import { clearTables, getSyncStatus } from '@rainlanguage/orderbook';

const db = createLocalDb(); // e.g., SQLite, IndexedDB, Dexie …
const dbCallback = (query: string, params?: unknown[]) => db.execute(query, params);

const setDbResult = client.setDbCallback(dbCallback);
if (setDbResult.error) throw new Error(setDbResult.error.readableMsg);

const syncResult = await client.syncLocalDatabase(dbCallback, status => console.log(status), 137);
if (syncResult.error) throw new Error(syncResult.error.readableMsg);

const syncStatus = await getSyncStatus(dbCallback);
if (!syncStatus.error) console.log(syncStatus.value);

const clearResult = await clearTables(dbCallback);
if (clearResult.error) throw new Error(clearResult.error.readableMsg);
```

The provided callbacks let you plug in whatever persistence layer makes sense (IndexedDB in browsers, SQLite in desktop apps, etc.), while the SDK drives schema creation and synchronization.

## Dotrain authoring & deployment flows

### Load remote strategies with `DotrainRegistry`

```ts
import { DotrainRegistry } from '@rainlanguage/orderbook';

const registryResult = await DotrainRegistry.new('https://example.com/registry.txt');
if (registryResult.error) throw new Error(registryResult.error.readableMsg);
const registry = registryResult.value;

const orderMenuResult = registry.getAllOrderDetails();
if (orderMenuResult.error) throw new Error(orderMenuResult.error.readableMsg);
const orderMenu = orderMenuResult.value; // Map<orderKey, { name, description, short_description }>

const deploymentsResult = registry.getDeploymentDetails('fixed-limit');
if (deploymentsResult.error) throw new Error(deploymentsResult.error.readableMsg);
const deployments = deploymentsResult.value;
```

Registry files follow the format:

```
https://example.com/shared-settings.yaml
fixed-limit https://example.com/orders/fixed-limit.rain
dca https://example.com/orders/dca.rain
```

The SDK merges the shared settings YAML with each order’s `.rain` content before you ever build a GUI.

### Build a deployment GUI

```ts
const guiResult = await registry.getGui(
  'fixed-limit',
  'flare-prod',
);
if (guiResult.error) throw new Error(guiResult.error.readableMsg);
const gui = guiResult.value;

const configResult = gui.getAllGuiConfig();
if (configResult.error) throw new Error(configResult.error.readableMsg);
const config = configResult.value;

const selectTokensResult = gui.getSelectTokens();
if (selectTokensResult.error) throw new Error(selectTokensResult.error.readableMsg);
const selectTokens = selectTokensResult.value;

const depositsResult = gui.getDeposits();
if (depositsResult.error) throw new Error(depositsResult.error.readableMsg);
const deposits = depositsResult.value;

await gui.setSelectToken('input-token', '0xUSDT');
const fieldResult = gui.setFieldValue('amount-per-trade', '50');
if (fieldResult.error) throw new Error(fieldResult.error.readableMsg);
await gui.setDeposit('input-token', '1000');
const vaultIdResult = gui.setVaultId('input', 'input-token', '42');
if (vaultIdResult.error) throw new Error(vaultIdResult.error.readableMsg);

const allowancesResult = await gui.checkAllowances('0xOwner');
if (allowancesResult.error) throw new Error(allowancesResult.error.readableMsg);
const allowances = allowancesResult.value;

const approvalCalldatasResult = await gui.generateApprovalCalldatas('0xOwner');
if (approvalCalldatasResult.error) throw new Error(approvalCalldatasResult.error.readableMsg);

const depositCalldatasResult = await gui.generateDepositCalldatas();
if (depositCalldatasResult.error) throw new Error(depositCalldatasResult.error.readableMsg);

const deploymentArgsResult = await gui.getDeploymentTransactionArgs('0xOwner');
if (deploymentArgsResult.error) throw new Error(deploymentArgsResult.error.readableMsg);
const { approvals, deploymentCalldata, orderbookAddress, chainId } = deploymentArgsResult.value;

const rainlangResult = await gui.getComposedRainlang();
if (rainlangResult.error) throw new Error(rainlangResult.error.readableMsg);
const composedRainlang = rainlangResult.value;

const serializedStateResult = gui.serializeState();
if (!serializedStateResult.error) {
  localStorage.setItem('fixed-limit-state', serializedStateResult.value);
}
```

Restore the workflow later with `DotrainOrderGui.newFromState(dotrainText, serializedState, callback)` if you want to bypass the registry fetch.

### Work directly with dotrain files

```ts
import fs from 'node:fs/promises';
import { DotrainOrder } from '@rainlanguage/orderbook';

const dotrainText = await fs.readFile('./orders/dca.rain', 'utf8');
const sharedSettingsYaml = await fs.readFile('./settings.yaml', 'utf8');

const dotrainResult = await DotrainOrder.create(dotrainText, [sharedSettingsYaml]);
if (dotrainResult.error) throw new Error(dotrainResult.error.readableMsg);
const dotrain = dotrainResult.value;

const scenarioResult = await dotrain.composeScenarioToRainlang('backtest');
if (!scenarioResult.error) console.log(scenarioResult.value);

const deploymentResult = await dotrain.composeDeploymentToRainlang('flare-prod');
if (!deploymentResult.error) console.log(deploymentResult.value);

const postTaskResult = await dotrain.composeScenarioToPostTaskRainlang('flare-prod');
if (!postTaskResult.error) console.log(postTaskResult.value);
```

## Utility exports

- `getOrderHash`, `keccak256`, `keccak256HexString` – deterministic hashing helpers for Rain orders or arbitrary payloads.
- `Float` – arbitrary-precision arithmetic with parsing, formatting, comparisons, math ops, fixed-decimal conversions, and helpers like `Float.zero()` or `.formatWithRange(...)`.
- `OrderbookYaml` – minimal helper for parsing YAML sources and looking up orderbook configs by address.
- `RaindexClient.getAllAccounts()` / `getAllVaultTokens()` – introspect accounts and ERC20 metadata defined in your YAML or discovered via subgraphs.
- `clearTables`, `getSyncStatus`, `RaindexClient.syncLocalDatabase`, `RaindexClient.setDbCallback` – plug in a persistent cache for offline apps.
- `RaindexVaultsList.getWithdrawCalldata()` – multicall builder that withdraws every vault with a balance.
- `RaindexOrder.convertToSgOrder()` – convert WASM order representations back into the raw subgraph schema when you need to interop with other tooling.

Type definitions are published in `dist/*/index.d.ts`; use them for richer TS inference in your apps.

## Error handling pattern

Every exported function returns a `WasmEncodedResult<T>`:

```ts
interface WasmEncodedResult<T> {
  value?: T;
  error?: {
    msg: unknown;
    readableMsg: string;
  };
}

const result = await client.getVault(14, '0xOrderbook', '0x01');
if (result.error) {
  console.error('Vault lookup failed:', result.error.readableMsg);
  return;
}
console.log(result.value);
```

## Contributing

This SDK is part of the Rain Language ecosystem. For contributions and issues, please visit the [GitHub repository](https://github.com/rainlanguage/rain.orderbook).
