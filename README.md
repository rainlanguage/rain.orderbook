# rain.orderbook

## Run Tauri App for local development
- Prepare the tauri build: `./prep-tauri.sh`
- Copy `tauri-app/.env.example` to `tauri-ap/.env` and fill in values
- Run tauri for develoment: `nix develop .#tauri-shell --command cargo tauri dev`

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

## Dev stuff

### Local environment & CI

Uses nixos.

Install `nix develop` - https://nixos.org/download.html.

Run `nix develop` in this repo to drop into the shell. Please ONLY use the nix
version of `foundry` for development, to ensure versions are all compatible.

Read the `flake.nix` file to find some additional commands included for dev and
CI usage.

## Legal stuff

Everything is under DecentraLicense 1.0 (DCL-1.0) which can be found in `LICENSES/`.

This is basically `CAL-1.0` which is an open source license
https://opensource.org/license/cal-1-0

The non-legal summary of DCL-1.0 is that the source is open, as expected, but
also user data in the systems that this code runs on must also be made available
to those users as relevant, and that private keys remain private.

Roughly it's "not your keys, not your coins" aware, as close as we could get in
legalese.

This is the default situation on permissionless blockchains, so shouldn't require
any additional effort by dev-users to adhere to the license terms.

This repo is REUSE 3.2 compliant https://reuse.software/spec-3.2/ and compatible
with `reuse` tooling (also available in the nix shell here).

```
nix develop -c rainix-sol-legal
```

## Contributions

Contributions are welcome **under the same license** as above.

Contributors agree and warrant that their contributions are compliant.