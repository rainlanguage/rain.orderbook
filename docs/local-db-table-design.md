# OrderBook Local Database Table Design

## Overview

This document explains the relationship between OrderBook contract events and the local database table structure. The local database system captures and processes events from the OrderBook smart contract to enable efficient offline analysis and querying.

## Contract Events Source

The events are sourced from the OrderBook V4 contract, specifically defined in:
- **Interface**: `lib/rain.orderbook.interface/src/interface/IOrderBookV4.sol`
- **Implementation**: `src/concrete/ob/OrderBook.sol`
- **Contract Address**: `0xd2938e7c9fe3597f78832ce780feb61945c377d7`

## Supported Events

The local database system processes the following 8 events from the OrderBook contract:

### 1. `Deposit` Event
```solidity
event Deposit(address sender, address token, uint256 vaultId, uint256 amount);
```
**Purpose**: Emitted when tokens are deposited into a vault
**Table**: `deposits`

### 2. `Withdraw` Event  
```solidity
event Withdraw(address sender, address token, uint256 vaultId, uint256 targetAmount, uint256 amount);
```
**Purpose**: Emitted when tokens are withdrawn from a vault
**Table**: `withdrawals`

### 3. `AddOrderV2` Event
```solidity
event AddOrderV2(address sender, bytes32 orderHash, OrderV3 order);
```
**Purpose**: Emitted when a new order is added to the orderbook
**Table**: `order_events` (with related `order_ios` table)

### 4. `RemoveOrderV2` Event
```solidity
event RemoveOrderV2(address sender, bytes32 orderHash, OrderV3 order);
```
**Purpose**: Emitted when an order is removed from the orderbook
**Table**: `order_events`

### 5. `TakeOrderV2` Event
```solidity
event TakeOrderV2(address sender, TakeOrderConfigV3 config, uint256 input, uint256 output);
```
**Purpose**: Emitted when an order is taken (market buy)
**Table**: `take_orders` (with related `take_order_contexts` and `context_values` tables)

### 6. `ClearV2` Event
```solidity
event ClearV2(address sender, OrderV3 alice, OrderV3 bob, ClearConfig clearConfig);
```
**Purpose**: Emitted before two orders are cleared against each other
**Table**: `clear_v2_events`

### 7. `AfterClear` Event
```solidity
event AfterClear(address sender, ClearStateChange clearStateChange);
```
**Purpose**: Emitted after two orders clear with final state changes
**Table**: `after_clear_events`

### 8. `MetaV1_2` Event
```solidity
event MetaV1_2(address sender, bytes32 subject, bytes meta);
```
**Purpose**: Emitted to store metadata associated with orders
**Table**: `meta_events`

## Table Design Rationale

### Core Design Principles

1. **Event-Driven Structure**: Each table corresponds directly to specific contract events
2. **Blockchain Metadata**: All tables include standard blockchain event metadata (block number, timestamp, transaction hash, log index)
3. **Unique Constraints**: Every table uses `(transaction_hash, log_index)` as a unique constraint to prevent duplicates
4. **Indexed Queries**: Strategic indexing on commonly queried fields (addresses, hashes, block numbers)
5. **Normalized Relations**: Complex events are broken into multiple related tables to avoid JSON storage

### Specific Table Designs

#### `deposits` & `withdrawals`
- **Why**: Simple vault operations with straightforward parameters
- **Design**: Direct mapping of event parameters to columns
- **Key Fields**: `sender`, `token`, `vault_id`, `amount`

#### `order_events` & `order_ios` 
- **Why**: Orders have complex structures with variable-length input/output arrays
- **Design**: Normalized approach - main event in `order_events`, I/O details in related `order_ios`
- **Key Fields**: `order_hash` (primary identifier), `event_type` (AddOrderV2/RemoveOrderV2)
- **Normalization**: Each valid input/output token becomes a row in `order_ios`

#### `take_orders`, `take_order_contexts` & `context_values`
- **Why**: Take orders can include signed context arrays with variable-length data
- **Design**: Three-table normalization to handle nested array structures
- **Key Fields**: `order_owner`, `order_nonce`, input/output amounts
- **Normalization**: Signed contexts → `take_order_contexts` → individual values in `context_values`

#### `clear_v2_events` & `after_clear_events`
- **Why**: Clearing involves two separate events that need to be correlated
- **Design**: Separate tables that can be joined by `(transaction_hash, block_number)`
- **Key Fields**: Alice/Bob order hashes, vault IDs, input/output amounts

#### `meta_events`
- **Why**: Metadata can be arbitrary bytes and is optional
- **Design**: Simple table storing raw metadata as BLOB
- **Key Fields**: `subject` (order hash), `meta` (raw bytes)

### String vs Integer Storage

**Why strings for large numbers?**
- Ethereum uses 256-bit integers that exceed most database integer types
- String storage prevents precision loss and overflow issues
- Vault IDs and amounts can be arbitrarily large
- Block numbers remain INTEGER as they fit comfortably in 64-bit integers

### Indexing Strategy

Indexes are created on:
- **Join keys**: For efficient table relationships 
- **Filter keys**: Common query patterns (by address, token, time range)
- **Sort keys**: Block numbers and timestamps for chronological queries
- **Lookup keys**: Order hashes, transaction hashes for direct access

## Event Processing Pipeline

1. **Fetch**: Raw events retrieved from blockchain RPC
2. **Decode**: Event topics and data decoded using Rust bindings  
3. **Transform**: Structured data prepared for SQL insertion
4. **Load**: SQL INSERT statements executed against SQLite database

This design enables efficient analysis of OrderBook activity including:
- Vault balance tracking over time
- Order lifecycle analysis (add → take/clear → remove)
- Trading volume and liquidity metrics
- User behavior patterns
- Cross-order arbitrage opportunities