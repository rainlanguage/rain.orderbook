{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/974d019731064cb9312d4ddab31c5b0939d4186d";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, flake-utils, rainix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};
      in {
        packages = rec {

          tauri-release-env = rainix.tauri-release-env.${system};

          ob-tauri-prelude = rainix.mkTask.${system} {
            name = "ob-tauri-prelude";
            body = ''
              set -euxo pipefail

              # Generate Typescript types from rust types
              mkdir -p tauri-app/src/typeshare;
              typeshare crates/subgraph/src/types/vault.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vault.ts;
              typeshare crates/subgraph/src/types/vaults.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaults.ts;
              typeshare crates/subgraph/src/types/order.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/order.ts;
              typeshare crates/subgraph/src/types/orders.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orders.ts;
              typeshare tauri-app/src-tauri/src/toast.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/toast.ts;
              typeshare tauri-app/src-tauri/src/transaction_status.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/transactionStatus.ts;

              # Fix linting of generated types
              cd tauri-app && npm i && npm run lint
            '';
            additionalBuildInputs = [
              pkgs.typeshare
            ];
          };
        } // rainix.packages.${system};
        devShells = rainix.devShells.${system};
      }
    );

}
