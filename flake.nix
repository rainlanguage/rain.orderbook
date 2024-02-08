{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, flake-utils, rainix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};
      in rec {
        packages = rec {

          tauri-release-env = rainix.tauri-release-env.${system};

          ob-tauri-prelude = rainix.mkTask.${system} {
            name = "ob-tauri-prelude";
            body = ''
              set -euxo pipefail

              # Generate Typescript types from rust types
              mkdir -p tauri-app/src/lib/typeshare;
              typeshare crates/subgraph/src/types/vault_detail.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultDetail.ts;
              typeshare crates/subgraph/src/types/vaults_list.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultsList.ts;
              typeshare crates/subgraph/src/types/vault_list_balance_changes.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultListBalanceChanges.ts;
              typeshare crates/subgraph/src/types/vault_balance_change.rs --lang=typescript --output-file=/tmp/vaultBalanceChange.ts;
              cat /tmp/vaultBalanceChange.ts >> tauri-app/src/lib/typeshare/vaultListBalanceChanges.ts;
              typeshare crates/subgraph/src/types/order_detail.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orderDetail.ts;
              typeshare crates/subgraph/src/types/orders_list.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/ordersList.ts;
              typeshare crates/subgraph/src/types/orders_list_for_vault.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/ordersListForVault.ts;
              typeshare tauri-app/src-tauri/src/toast.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/toast.ts;
              typeshare tauri-app/src-tauri/src/transaction_status.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/transactionStatus.ts;

              # Fix linting of generated types
              cd tauri-app && npm i && npm run lint
            '';
            additionalBuildInputs = [
              pkgs.typeshare
              pkgs.wasm-bindgen-cli
              rainix.rust-toolchain.${system}
              rainix.rust-build-inputs.${system}
            ];
          };
          ob-tauri-test =  rainix.mkTask.${system} {
            name = "ob-tauri-test";
            body = ''
              set -euxo pipefail

              cd tauri-app && npm i && npm run test
            '';
          };
        } // rainix.packages.${system};

        devShells.default = rainix.devShells.${system}.default;
        devShells.tauri-shell = pkgs.mkShell {
          packages = [
            packages.ob-tauri-prelude
            packages.ob-tauri-test
          ];
          inputsFrom = [ rainix.devShells.${system}.tauri-shell ];
        };

      }
    );

}
