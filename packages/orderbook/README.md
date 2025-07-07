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

```javascript
import { parseYaml, RaindexClient } from '@rainlanguage/orderbook';

// Parse configuration
const result = parseYaml([orderbookYaml, dotrainYaml], true);
if (result.error) {
  console.error("Parse failed:", result.error.readableMsg);
  return;
}

// Get a single order
const raindexClient = RaindexClient.getOrders();
if (raindexClient.error) {
  console.error("Failed to get raindex client:", raindexClient.error.readableMsg);
  return;
}
const order = raindexClient.getOrderByHash(1, "0x...", "0x...");
if (order.error) {
  console.error("Failed to get order:", order.error.readableMsg);
  return;
}
console.log("Order:", order.value);
```

## Contributing

This SDK is part of the Rain Language ecosystem. For contributions and issues, please visit the [GitHub repository](https://github.com/rainlanguage/rain.orderbook).
