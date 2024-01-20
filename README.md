# rain.orderbook

Docs at https://rainprotocol.github.io/foundry.template

## Use as template

```
forge init -t rainprotocol/foundry.template <projectname>
cd <projectname>
forge install foundry-rs/forge-std
```

Then update the readme, set the docs url and configure github pages on github repo settings.

For CI deployments, setup all the environment variables and define contracts to
deploy in the matrix.

## Generating Rust Types from Graphql

**NOTICE** The crates and tauri app are currently expecting a subgraph compatible with the schema in `crates/subgraph/schema/orderbook.graphql`. 
When this project is migrated to use the subgraph schema defined in `./subgraph/schema/graphql`, the command below will change.

1. Run the following commands to generate Rust types from GraphQL Queries
```bash
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/vault.graphql  > crates/subgraph/src/types/vault.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/vaults.graphql  > crates/subgraph/src/types/vaults.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/order.graphql  > crates/subgraph/src/types/order.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/orders.graphql  > crates/subgraph/src/types/orders.rs
```

2. Prepend each generated types file with the following:
```rust
use crate::schema;
use typeshare::typeshare;
```

3. Add the following derives for all generated Rust types:
```rust
#[derive(Clone)]
```

4. Add the following derives for all generated Rust types that also derive `cynic::QueryFragment`:
```rust
#[derive(Serialize)]
```

5. Add the following macros to all generated Rust types
```rust
#[typeshare]
```

## Generating Typescript Types from Rust Types

Run the following to generate Typescript types from Rust types in `src/types`.
```bash
typeshare crates/subgraph/src/types/vault.rs --lang=typescript --output-file=tauri-app/src/types/vault.ts
typeshare crates/subgraph/src/types/vaults.rs --lang=typescript --output-file=tauri-app/src/types/vaults.ts
typeshare crates/subgraph/src/types/order.rs --lang=typescript --output-file=tauri-app/src/types/order.ts
typeshare crates/subgraph/src/types/orders.rs --lang=typescript --output-file=tauri-app/src/types/orders.ts
```

