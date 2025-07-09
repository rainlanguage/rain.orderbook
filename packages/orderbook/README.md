# Raindex SDK

A TypeScript/JavaScript SDK for interacting with orderbook contracts, providing comprehensive functionality for order management, configuration parsing, and blockchain interactions.

## Overview

This SDK provides Rust-powered WebAssembly bindings for orderbook functionality, enabling developers to:

- Parse and validate YAML configuration files
- Generate order hashes and calldata
- Interact with orderbook contracts
- Handle complex order operations and quotes

## Installation

```bash
npm install @rainlanguage/orderbook
```

## Quick Start

This quickstart guide will show you how to use the SDK to do the following:

- Parse a YAML configuration file
- Get RaindexClient instance to interact with the orderbook
- Get all orders
- Get a single order by hash
- Get all vaults
- Get a single vault
- Get calldatas for vault deposit and withdraw
- Get DotrainOrderGui instance to construct UI to deploy an order
- Deploy an order using the DotrainOrderGui instance

### Parse YAML configuration

```javascript
import { parseYaml } from '@rainlanguage/orderbook';

// Prepare YAML configuration
// The latest version can be found in our repository
// https://github.com/rainlanguage/rain.strategies
const YAML = `
  version: 2
  networks:
    flare:
  ...
`

// Parse configuration
// Multiple YAML files can be passed
const result = parseYaml([YAML]);
if (result.error) {
  console.error("Parse failed:", result.error.readableMsg);
  return;
}
const config = result.value;

// Get dotrain order config
// dotrainOrder: {
//   orders: {},
//   scenarios: {},
//   charts: {},
//   deployments: {},
// },
config.dotrainOrder

// Get orderbook config
// orderbook: {
//   version: '2',
//   networks: {},
//   subgraphs: {},
//   metaboards: {},
//   orderbooks: {},
//   accounts: {},
//   tokens: {},
//   deployers: {},
// },
config.orderbook
```

### Get raindex client to interact with the orderbook

```javascript
import { RaindexClient } from '@rainlanguage/orderbook';

// Pass in the YAML configuration to get a raindex client
const raindexClient = RaindexClient.new([YAML])
```

### Get all orders

```javascript
// Get all orders for all orderbooks in all networks
const result = await raindexClient.getOrders();
if (orders.error) {
  console.error("Failed to get orders:", orders.error.readableMsg);
  return;
}
const orders = result.value;

// Get all orders for all orderbooks in selected networks
// Ethereum and Polygon are selected using their chain ids
const result = await raindexClient.getOrders([1, 137]);
if (orders.error) {
  console.error("Failed to get orders:", orders.error.readableMsg);
  return;
}
const orders = result.value;
```

### Get a single order by hash

```javascript
// Get a single order by hash
// Pass in the chain id, orderbook address, and order hash
const result = await raindexClient.getOrderByHash(1, "0x...", "0x...");
if (result.error) {
  console.error("Failed to get order:", result.error.readableMsg);
  return;
}
const order = result.value;

// Orders are instances of RaindexOrder
// Each order has various getters and functions
// Here are some examples

// Get the composed Rainlang for this order
order.rainlang

// Get all the vaults for this order
order.vaults

// Get calldata to remove this order from the orderbook
await order.getRemoveCalldata()

// Get all the trades for this order
await order.getTradesList()
```

### Get all vaults

```javascript
// Get all vaults for all orderbooks in all networks
const result = await raindexClient.getVaults();
if (vaults.error) {
  console.error("Failed to get vaults:", vaults.error.readableMsg);
  return;
}
const vaults = result.value;

// Get all orders for all orderbooks in selected networks
// Ethereum and Polygon are selected using their chain ids
const result = await raindexClient.getVaults([1, 137]);
if (vaults.error) {
  console.error("Failed to get vaults:", vaults.error.readableMsg);
  return;
}
const vaults = result.value;
```

### Get a single vault

```javascript
// Get a single vault by its id
// Pass in the chain id, orderbook address, and vault id
const result = await raindexClient.getVault(1, "0x...", "0x...");
if (vault.error) {
  console.error("Failed to get vault:", vault.error.readableMsg);
  return;
}
const vault = result.value;

// Vaults are instances of RaindexVault
// Each vault has various getters and functions
// Here are some examples

// Get the token for this vault
vault.token

// Get the balance for this vault
vault.balance

// Get the orders this vault is used as input and output for
vault.ordersAsInput
vault.ordersAsOutput

// Get vault balance changes
await vault.getBalanceChanges()
```

### Get calldatas for vault deposit and withdraw

```javascript
// Get the vault using the raindex client
const vault = await raindexClient.getVault(1, "0x...", "0x...");

// Get calldata to deposit tokens into this vault
// Amount is in token's smallest unit (e.g., "1000000000000000000" for 1 token with 18 decimals)
await vault.getDepositCalldata("1000000000000000000")

// Get calldata to withdraw tokens from this vault
// Amount is in token's smallest unit (e.g., "1000000000000000000" for 1 token with 18 decimals)
await vault.getWithdrawCalldata("1000000000000000000")
```

### Get DotrainOrderGui instance to construct UI to deploy an order

```javascript
import { DotrainOrderGui } from '@rainlanguage/orderbook';

// Prepare the dotrain
// Various strategies can be found in our repository
// https://github.com/rainlanguage/rain.strategies
const dotrain = `
  version: 2
  networks:
    flare:
  ...

  ---
  
  #calculate-io:
  ...
`

// Get details for all deployments
// Use this to build your own UI to select a deployment
const result = await DotrainOrderGui.getDeploymentDetails(dotrain)
if (result.error) {
  console.error("Failed to get deployment details:", result.error.readableMsg);
  return;
}
const deployments = result.value;

// Get the DotrainOrderGui instance for a given deployment
const result = await DotrainOrderGui.newWithDeployment(dotrain, selectedDeployment)
if (result.error) {
  console.error("Failed to get DotrainOrderGui instance:", result.error.readableMsg);
  return;
}
const gui = result.value;

// Gui is an instance of DotrainOrderGui
// You can use the gui to fully control the order deployment process

// Get the tokens that need to be selected before deploying the order
const result = gui.getSelectTokens()
if (result.error) {
  console.error("Failed to get select tokens:", result.error.readableMsg);
  return;
}
const selectTokens = result.value;
// Set the tokens that are needed for the deployment
gui.setSelectToken(selectTokens[0].key, "0x...")

// Get all the configuration needed to construct the UI
const result = gui.getAllGuiConfig()
if (result.error) {
  console.error("Failed to get all gui config:", result.error.readableMsg);
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
} = result.value;

// Set the field values for the deployment
gui.setFieldValue("fixed-io", "1000000000000000000")

// Set the deposit amount for a token
gui.setDeposit("input-token", "1000000000000000000")

// Set a custom vaultId for an input vault
// This is the first input vault in the order
gui.setVaultId(true, 0, "42")
```

### Deploy an order using the DotrainOrderGui instance

```javascript
// After setting all the necessary values in the gui instance
// we can generate the calldatas to deploy the order

// Pass the owner address for the deployment
const result = await gui.getDeploymentTransactionArgs("0x...")
if (result.error) {
  console.error("Failed to get deployment transaction args:", result.error.readableMsg);
  return;
}
const {
  // List of calldatas to approve tokens for the deployment
  approvals,
  // Calldata to deposit tokens (if any) and add the order to the orderbook
  deploymentCalldata,
} = result.value;
```

## Contributing

This SDK is part of the Rain Language ecosystem. For contributions and issues, please visit the [GitHub repository](https://github.com/rainlanguage/rain.orderbook).
