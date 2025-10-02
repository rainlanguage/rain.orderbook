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

This quickstart guide will show you how to use the SDK to:

- Parse a YAML configuration file
- Get RaindexClient instance to interact with the orderbook
- Get all orders
- Get a single order by hash
- Get all vaults
- Get a single vault
- Get calldatas for vault deposit and withdraw
- Get DotrainOrderGui instance to construct UI to deploy an order
- Deploy an order using the DotrainOrderGui instance

Make sure to also check out our [specs repository](https://github.com/rainlanguage/specs) for more details on how configuration files, rainlang, and the orderbook work.

### Parse YAML configuration

```javascript
import { parseYaml } from '@rainlanguage/orderbook';

// Prepare YAML configuration. This configuration file is used by the raindex client
// to do various read operations on the orderbooks configured in the YAML file.
// It is also used to generate the calldatas to do operations on orders and vaults.
// The latest version we use in our applications can be found in our repository
// https://github.com/rainlanguage/rain.strategies/blob/main/settings.yaml
const YAML = `
  version: 2
  networks:
    flare:
      rpc: <Flare RPC URL>
      chain-id: 14
      network-id: 14
      currency: FLR
  subgraphs:
    flare: <Flare Subgraph URL>
  orderbooks:
    flare:
      address: <Orderbook address on Flare>
      network: flare
      subgraph: flare
  tokens:
    flare-usdt:
      network: flare
      address: <Token address for USDT on Flare>
      decimals: 6
      label: USDT
      symbol: USDT
`;

// Parse configuration files. Multiple YAML files can be passed in an array.
// These files are merged together to form a single configuration object
// The rules for the merge are defined in the specs repository
const result = parseYaml([YAML]);
if (result.error) {
  console.error("Parse failed:", result.error.readableMsg);
  // Handle error appropriately in your application
  return;
}
const config = result.value;

// Dotrain order config contains order-related configurations including orders, scenarios, deployments, GUI settings, and charts.
// It handles the dotrain-specific configuration for order management, testing scenarios, and visualization.
// Structure: { orders: {}, scenarios: {}, charts: {}, deployments: {}, gui: {} }
const dotrainConfig = config.dotrainOrder;

// Orderbook config contains the broader orderbook infrastructure including networks, subgraphs, metaboards, orderbooks, tokens, deployers and accounts.
// It manages the underlying blockchain and service configurations that support the orderbook.
// Structure: { version: string, networks: {}, subgraphs: {}, metaboards: {}, orderbooks: {}, accounts: {}, tokens: {}, deployers: {} }
const orderbookConfig = config.orderbook;
```

### Get raindex client to interact with the orderbook

```javascript
import { RaindexClient } from '@rainlanguage/orderbook';

// Pass in the YAML configuration to get a raindex client.
// This client is the main entry point to access orders and vaults in the orderbooks that are configured in the YAML file.
// Various functions are available to build the UI for listing orders and vaults.
const raindexClient = RaindexClient.new([YAML]);

// The client automatically connects to the configured subgraphs and RPCs
// No additional initialization is required
```

### Get all orders

```javascript
// Get all orders for all orderbooks in all networks
// Returns an array of RaindexOrder instances
const result = await raindexClient.getOrders();
if (result.error) {
  console.error("Failed to get orders:", result.error.readableMsg);
  return;
}
const orders = result.value; // RaindexOrder[]

// Get all orders for all orderbooks in selected networks
// Chain IDs reference: Ethereum (1), Polygon (137), Arbitrum (42161), Base (8453), Flare (14)
const selectedNetworks = [1, 137]; // Ethereum and Polygon
const filteredResult = await raindexClient.getOrders(selectedNetworks);
if (filteredResult.error) {
  console.error("Failed to get orders:", filteredResult.error.readableMsg);
  return;
}
const filteredOrders = filteredResult.value;

// Each order contains:
// - id: unique identifier
// - orderHash: the keccak256 hash of the order
// - owner: address that deployed the order
// - inputs: vaults for tokens that can be sold by the order
// - outputs: vaults for tokens that can be bought by the order
// - vaults: all the vaults combined from inputs and outputs
// More details can be found in the RaindexOrder class documentation
```

### Get a single order by hash

```javascript
// Get a single order by hash
// Parameters:
// - chainId: The network chain ID (e.g., 1 for Ethereum, 137 for Polygon)
// - orderbookAddress: The orderbook contract address (e.g., "0x59401C93239a3D8956C7881f0dB45B5727241872")
// - orderHash: The order hash (e.g., "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12")
const chainId = 14; // Flare network
const orderbookAddress = "0x59401C93239a3D8956C7881f0dB45B5727241872";
const orderHash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12";

const result = await raindexClient.getOrderByHash(chainId, orderbookAddress, orderHash);
if (result.error) {
  console.error("Failed to get order:", result.error.readableMsg);
  return;
}
const order = result.value; // RaindexOrder instance

// Orders are instances of RaindexOrder
// Each order has various getters and functions
// Here are some examples:

// Get the composed Rainlang expression for this order
const rainlangExpression = order.rainlang; // string

// Get all the vaults associated with this order
const orderVaultsList = order.vaultsList; // RaindexVaultsList
const orderVaults = orderVaultsList.items; // RaindexVault[]

// Get calldata to remove this order from the orderbook
// Returns hex-encoded calldata that can be sent to the orderbook contract
const removeCalldata = await order.getRemoveCalldata(); // string (hex)

// Get all the trades for this order
// Returns historical trade data from the subgraph
const trades = await order.getTradesList(); // Trade[]

// More methods and properties are available in the RaindexOrder class
```

### Order inputs / outputs / vaults

An `Order` exposes a `RaindexVaultsList` wrapper around the underlying `RaindexVault[]`.
The list offers batch operations (e.g. multicall-withdraw).
You can still access the raw vaults via the `items` getter:

```javascript
const orderVaultsList = order.vaultsList; // RaindexVaultsList
const orderVaults = orderVaultsList.items; // RaindexVault[]
```

The `RaindexOrder` exposes the following getters that return a `RaindexVaultsList`:

- `vaultsList`
- `inputsList`
- `outputsList`
- `inputsOutputsList`

### Get all vaults

```javascript
// Get all vaults for all orderbooks in all networks
// Vaults are used to hold tokens for orders
const result = await raindexClient.getVaults();
if (result.error) {
  console.error("Failed to get vaults:", result.error.readableMsg);
  return;
}
const vaults = result.value; // RaindexVault[]

// Get all vaults for all orderbooks in selected networks
// Chain IDs reference: Ethereum (1), Polygon (137), Arbitrum (42161), Base (8453), Flare (14)
const selectedNetworks = [1, 137]; // Ethereum and Polygon
const filteredResult = await raindexClient.getVaults(selectedNetworks);
if (filteredResult.error) {
  console.error("Failed to get vaults:", filteredResult.error.readableMsg);
  return;
}
const filteredVaults = filteredResult.value;

// Each vault contains:
// - id: unique vault identifier
// - token: the token held in this vault
// - balance: current balance in the vault
// - owner: address that owns the vault
// More details can be found in the RaindexVault class documentation
```

### Get a single vault

```javascript
// Get a single vault by its id
// Parameters:
// - chainId: The network chain ID (e.g., 1 for Ethereum, 137 for Polygon)
// - orderbookAddress: The orderbook contract address (e.g., "0x59401C93239a3D8956C7881f0dB45B5727241872")
// - vaultId: The vault ID (hex string, e.g., "0x01" or "0x2a")
const chainId = 14; // Flare network
const orderbookAddress = "0x59401C93239a3D8956C7881f0dB45B5727241872";
const vaultId = "0x01"; // Vault ID 1 in hex

const result = await raindexClient.getVault(chainId, orderbookAddress, vaultId);
if (result.error) {
  console.error("Failed to get vault:", result.error.readableMsg);
  return;
}
const vault = result.value; // RaindexVault instance

// Vaults are instances of RaindexVault
// Each vault has various getters and functions
// Here are some examples:

// Get the token information for this vault
const token = vault.token; // RaindexVaultToken

// Get the current balance for this vault in Float format
const balance = vault.balance; // Float

// Get the current balance in human-readable format
const formattedBalance = vault.formattedBalance; // string (e.g., "1")

// Get the orders that use this vault as input (selling from this vault)
const inputOrders = vault.ordersAsInput; // RaindexOrderAsIO[]

// Get the orders that use this vault as output (buying into this vault)
const outputOrders = vault.ordersAsOutput; // RaindexOrderAsIO[]

// Get vault balance change history
const balanceChanges = await vault.getBalanceChanges(); // RaindexVaultBalanceChange[]

// More methods and properties are available in the RaindexVault class
```

### Get calldatas for vault deposit and withdraw

```javascript
// Get the vault using the raindex client
const result = await raindexClient.getVault(14, "0x59401C93239a3D8956C7881f0dB45B5727241872", "0x01");
if (result.error) {
  console.error("Failed to get vault:", result.error.readableMsg);
  return;
}
const vault = result.value; // RaindexVault instance

// Get calldata to deposit tokens into this vault
const depositCalldata = await vault.getDepositCalldata("10.5");
// Returns hex-encoded calldata to be sent to the orderbook contract

// Get calldata to withdraw tokens from this vault
const withdrawCalldata = await vault.getWithdrawCalldata("5.25");
// Returns hex-encoded calldata to be sent to the orderbook contract
```

### Get DotrainOrderGui instance to construct UI to deploy an order

```javascript
import { DotrainOrderGui } from '@rainlanguage/orderbook';

// Prepare the dotrain that will be used for this order
// Various orders can be found in our repository. These are the ones we show in our webapp
// https://github.com/rainlanguage/rain.strategies/tree/main/src
const dotrain = `
  <other configurations are written here>

  gui:
    name: DCA Order
    description: Dollar cost averaging order for buying tokens over time
    deployments:
      flare-dca-eth:
        name: DCA into ETH on Flare
        description: Buy ETH with USDT using dollar cost averaging
        deposits:
          - token: input-token
            presets:
              - 1000
              - 5000
              - 10000
        fields:
          - binding: amount-per-trade
            name: Amount per trade
            description: USDT amount to spend per trade
            presets:
              - value: 10
              - value: 50
              - value: 100
          - binding: frequency
            name: Trade frequency
            description: Hours between trades
            presets:
              - name: Every hour
                value: 1
              - name: Every day
                value: 24
              - name: Every week
                value: 168
        select-tokens:
          - key: input-token
            label: USDT
          - key: output-token
            label: ETH

  ---

  #calculate-io:
  <your order logic goes here>
`;

// Get details for all deployments defined in the dotrain
// Use this to build your own UI to select a deployment
const deploymentResult = await DotrainOrderGui.getDeploymentDetails(dotrain);
if (deploymentResult.error) {
  console.error("Failed to get deployment details:", deploymentResult.error.readableMsg);
  return;
}
const deployments = deploymentResult.value;
// Returns array of deployments with their metadata:
// [{ key: "flare-dca-eth", name: "DCA into ETH on Flare", description: "..." }]

// Get the DotrainOrderGui instance for a specific deployment
const selectedDeploymentKey = "flare-dca-eth"; // From deployments[0].key
const guiResult = await DotrainOrderGui.newWithDeployment(dotrain, selectedDeploymentKey);
if (guiResult.error) {
  console.error("Failed to get DotrainOrderGui instance:", guiResult.error.readableMsg);
  return;
}
const gui = guiResult.value;

// Gui is an instance of DotrainOrderGui
// You can use the gui to fully control the order deployment process

// Get the tokens that need to be selected before deploying the order
const selectTokensResult = gui.getSelectTokens();
if (selectTokensResult.error) {
  console.error("Failed to get select tokens:", selectTokensResult.error.readableMsg);
  return;
}
const selectTokens = selectTokensResult.value;
// Returns tokens that need addresses set:
// [{ key: "input-token", label: "USDT" }, { key: "output-token", label: "ETH" }]

// Set the token addresses for the deployment
gui.setSelectToken("input-token", "0x96B41289D90444B8ADD57e6F265DB5aE8651DF29"); // USDT on Flare
gui.setSelectToken("output-token", "0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d"); // WFLR on Flare

// Get all the configuration needed to construct the UI
const configResult = gui.getAllGuiConfig();
if (configResult.error) {
  console.error("Failed to get all gui config:", configResult.error.readableMsg);
  return;
}
const {
  // All the field definitions that don't have defaults set in the YAML
  fieldDefinitionsWithoutDefaults,
  // All the field definitions that have defaults set in the YAML
  // These fields don't need to be set by the user
  fieldDefinitionsWithDefaults,
  // All the tokens that are defined as deposits in the YAML
  deposits,
  // All the input and output vault tokens
  // These can be used to define custom vaultIds for input and output vaults
  orderInputs,
  orderOutputs,
} = configResult.value;

// Set the field values for the deployment
// Field names come from the "binding" property in the dotrain configuration
gui.setFieldValue("amount-per-trade", "50"); // 50 USDT per trade
gui.setFieldValue("frequency", "24"); // Trade every 24 hours

// Set the deposit amount for a token
// Token names come from the deposits configuration
gui.setDeposit("input-token", "100.65");

// Set a custom vaultId for an input vault (optional)
// Parameters:
// - isInput: true for input vaults, false for output vaults
// - index: vault index (0 for first vault, 1 for second, etc.)
// - vaultId: custom vault ID as string
gui.setVaultId(true, 0, "42"); // Set custom vault ID 42 for the first input vault
```

### Deploy an order using the DotrainOrderGui instance

```javascript
// After setting all the necessary values in the gui instance
// we can generate the calldatas to deploy the order

// Pass the owner address for the deployment
// This will be the address that owns the order and can remove it later
const ownerAddress = "0x1234567890123456789012345678901234567890"; // Your wallet address

const deploymentArgsResult = await gui.getDeploymentTransactionArgs(ownerAddress);
if (deploymentArgsResult.error) {
  console.error("Failed to get deployment transaction args:", deploymentArgsResult.error.readableMsg);
  return;
}
const {
  // List of calldatas to approve tokens for the deployment
  // Each approval includes the token address and spender address
  approvals, // Array<{ token: string, spender: string, calldata: string }>
  // Calldata to deposit tokens (if any) and add the order to the orderbook
  deploymentCalldata, // string (hex-encoded calldata)
  // Optional calldata to publish metadata to the metaboard
  emitMetaCall, // { to: string, calldata: string } | undefined
  // Target orderbook and chain metadata
  orderbookAddress,
  chainId
} = deploymentArgsResult.value;
```

## API Reference

For detailed API documentation, see the [TypeDoc documentation](https://rainlanguage.github.io/rain.orderbook).

## Error Handling

All SDK methods return a `WasmEncodedResult<T>` object with the following structure:

```typescript
interface WasmEncodedResult<T> {
  value?: T;
  error?: {
    msg: any;
    readableMsg: string;
  };
}
```

Always check for errors before using the value:

```javascript
const result = await someMethod();
if (result.error) {
  console.error("Error:", result.error.readableMsg);
  // Handle error appropriately
  return;
}
// Safe to use result.value
```

## Contributing

This SDK is part of the Rain Language ecosystem. For contributions and issues, please visit the [GitHub repository](https://github.com/rainlanguage/rain.orderbook).
