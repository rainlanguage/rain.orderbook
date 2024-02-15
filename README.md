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
When this project is migrated to use the subgraph schema defined in `subgraph/schema/graphql`, the command below will change.

1. Run the following commands to generate Rust types from GraphQL Queries
```bash
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/vaultDetail.graphql  > crates/subgraph/src/types/vault_detail.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/vaultsList.graphql  > crates/subgraph/src/types/vaults_list.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/orderDetail.graphql  > crates/subgraph/src/types/order_detail.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/ordersList.graphql  > crates/subgraph/src/types/orders_list.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/vaultBalanceChangesList.graphql  > crates/subgraph/src/types/vault_balance_changes_list.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/orderClearsList.graphql  > crates/subgraph/src/types/order_clears_list.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/orderTakesList.graphql  > crates/subgraph/src/types/order_takes_list.rs
cynic querygen --schema crates/subgraph/schema/orderbook.graphql --query crates/subgraph/queries/orderTakeDetail.graphql  > crates/subgraph/src/types/order_take_detail.rs
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

6. Rename the conflicting enum `TokenVaultOrderBy::VaultId` to `TokenVaultOrderBy::VaultId2` in `crates/subgraphs/src/types/vaults.rs`

## Generating Typescript Types from Rust Types

Run the following from the repo root, outside the nix shell, to generate Typescript types from Rust types in `crates/subgraph/src/types`.
```bash
nix run .#ob-tauri-prelude
```

