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
      in {
        packages = rec {
          ob-tauri-prelude = rainix.mkTask.${system} {
            name = "ob-tauri-prelude";
            body = ''
              set -euxo pipefail

              # Generate Typescript types from rust types
              mkdir tauri-app/src/types;
              typeshare crates/subgraph/src/types/vault.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vault.ts;
              typeshare crates/subgraph/src/types/vaults.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaults.ts;
              typeshare crates/subgraph/src/types/order.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/order.ts;
              typeshare crates/subgraph/src/types/orders.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orders.ts;
            '';
          };
        } // rainix.packages.${system};
        devShells = rainix.devShells.${system};
      }
    );

}
