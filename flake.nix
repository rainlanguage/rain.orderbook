{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainprotocol/rainix/ba8f6e43d9a07722044a1167ed2069b06f6a9640";
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

              typeshare crates/subgraph/src/types/vault_balance_changes_list.rs  crates/subgraph/src/types/vault_balance_change.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultBalanceChangesList.ts;
              typeshare crates/subgraph/src/types/order_detail.rs crates/common/src/types/order_detail_extended.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orderDetail.ts;

              typeshare crates/subgraph/src/types/vault_detail.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultDetail.ts;
              typeshare crates/subgraph/src/types/vaults_list.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/vaultsList.ts;
              typeshare crates/subgraph/src/types/orders_list.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/ordersList.ts;
              typeshare crates/subgraph/src/types/order_takes_list.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orderTakesList.ts;
              typeshare crates/subgraph/src/types/order_take_detail.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orderTakeDetail.ts;

              typeshare crates/common/src/fuzz/mod.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/fuzz.ts;

              typeshare crates/settings/src/parse.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/appSettings.ts;
              typeshare crates/settings/src/config.rs crates/settings/src/chart.rs crates/settings/src/deployer.rs crates/settings/src/deployment.rs crates/settings/src/network.rs crates/settings/src/order.rs crates/settings/src/orderbook.rs crates/settings/src/scenario.rs crates/settings/src/token.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/config.ts;

              typeshare tauri-app/src-tauri/src/toast.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/toast.ts;
              typeshare tauri-app/src-tauri/src/transaction_status.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/transactionStatus.ts;

              node tauri-app/src/scripts/typeshareFix.cjs

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

          ob-tauri-before-build = rainix.mkTask.${system} {
            name = "ob-tauri-before-build";
            body = ''
              set -euxo pipefail

              npm i && npm run build

              rm -rf lib
              mkdir -p lib

              if [ ${if pkgs.stdenv.isDarwin then "1" else "0" } -eq 1 ]; then
                cp ${pkgs.libiconv}/lib/libcharset.1.dylib lib/libcharset.1.dylib
                chmod +w lib/libcharset.1.dylib
                install_name_tool -id @executable_path/../Frameworks/libcharset.1.dylib lib/libcharset.1.dylib
                otool -L lib/libcharset.1.dylib

                cp ${pkgs.libiconv}/lib/libiconv-nocharset.dylib lib/libiconv-nocharset.dylib
                chmod +w lib/libiconv-nocharset.dylib
                install_name_tool -id @executable_path/../Frameworks/libiconv-nocharset.dylib lib/libiconv-nocharset.dylib
                otool -L lib/libiconv-nocharset.dylib

                cp ${pkgs.libiconv}/lib/libiconv.dylib lib/libiconv.dylib
                chmod +w lib/libiconv.dylib
                install_name_tool -id @executable_path/../Frameworks/libiconv.dylib lib/libiconv.dylib
                install_name_tool -change ${pkgs.libiconv}/lib/libiconv-nocharset.dylib @executable_path/../Frameworks/libiconv-nocharset.dylib lib/libiconv.dylib
                install_name_tool -change ${pkgs.libiconv}/lib/libcharset.1.dylib @executable_path/../Frameworks/libcharset.1.dylib lib/libiconv.dylib
                otool -L lib/libiconv.dylib

                cp ${pkgs.gettext}/lib/libintl.8.dylib lib/libintl.8.dylib
                chmod +w lib/libintl.8.dylib
                install_name_tool -id @executable_path/../Frameworks/libintl.8.dylib lib/libintl.8.dylib
                install_name_tool -change ${pkgs.libiconv}/lib/libiconv.dylib @executable_path/../Frameworks/libiconv.dylib lib/libintl.8.dylib
                otool -L lib/libintl.8.dylib
              fi
            '';
          };

          ob-tauri-before-bundle = rainix.mkTask.${system} {
            name = "ob-tauri-before-bundle";
            body = ''
              set -euxo pipefail

              ls src-tauri/target/release

              if [ ${if pkgs.stdenv.isDarwin then "1" else "0" } -eq 1 ]; then
                install_name_tool -change ${pkgs.libiconv}/lib/libiconv.dylib @executable_path/../Frameworks/libiconv.dylib src-tauri/target/release/Rain\ Orderbook
                install_name_tool -change ${pkgs.gettext}/lib/libintl.8.dylib @executable_path/../Frameworks/libintl.8.dylib src-tauri/target/release/Rain\ Orderbook

                otool -L src-tauri/target/release/Rain\ Orderbook
                grep_exit_code=0
                otool -L src-tauri/target/release/Rain\ Orderbook | grep -q /nix/store || grep_exit_code=$?
                if [ $grep_exit_code -eq 0 ]; then
                  exit 1
                fi
              fi
            '';
          };
        } // rainix.packages.${system};

        devShells.default = rainix.devShells.${system}.default;
        devShells.tauri-shell = pkgs.mkShell {
          packages = [
            packages.ob-tauri-prelude
            packages.ob-tauri-test
            packages.ob-tauri-before-build
            packages.ob-tauri-before-bundle
          ];
          shellHook = rainix.devShells.${system}.tauri-shell.shellHook;
          buildInputs = rainix.devShells.${system}.tauri-shell.buildInputs ++ [pkgs.clang-tools];
          nativeBuildInputs = rainix.devShells.${system}.tauri-shell.nativeBuildInputs;
        };

      }
    );

}
