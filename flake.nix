{
  description = "Flake for development workflows.";

  inputs = {
    rainix.url = "github:rainlanguage/rainix";
    rain.url = "github:rainlanguage/rain.cli";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {self, flake-utils, rainix, rain }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = rainix.pkgs.${system};
      in rec {
        packages = rec {

          tauri-release-env = rainix.tauri-release-env.${system};

          raindex-prelude = rainix.mkTask.${system} {
            name = "raindex-prelude";
            body = ''
              set -euxo pipefail

              mkdir -p meta;
              forge script --silent ./script/BuildAuthoringMeta.sol;
              rain meta build \
                -i <(cat ./meta/OrderBookSubParserAuthoringMeta.rain.meta) \
                -m authoring-meta-v2 \
                -t cbor \
                -e deflate \
                -l none \
                -o meta/OrderBookSubParser.rain.meta \
                ;
            '';
          };

          ob-rs-test = rainix.mkTask.${system} {
            name = "ob-rs-test";
            body = ''
              set -euxo pipefail
              cargo test --workspace
            '';
          };

          tauri-rs-test = rainix.mkTask.${system} {
            name = "tauri-rs-test";
            body = ''
              set -euxo pipefail
              cd tauri-app/src-tauri 
              cargo test
            '';
          };

          ob-tauri-prelude = rainix.mkTask.${system} {
            name = "ob-tauri-prelude";
            body = ''
              set -euxo pipefail

              # Generate Typescript types from rust types
              mkdir -p tauri-app/src/lib/typeshare

              export CARGO_HOME=$(mktemp -d)
              cargo install --git https://github.com/tomjw64/typeshare --rev 556b44aafd5304eedf17206800f69834e3820b7c
              export PATH=$PATH:$CARGO_HOME/bin

              typeshare crates/subgraph/src/types/common.rs crates/subgraph/src/types/order.rs crates/subgraph/src/types/vault.rs crates/subgraph/src/types/order_trade.rs crates/common/src/types/order_detail_extended.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/subgraphTypes.ts;

              typeshare crates/settings/src/parse.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/appSettings.ts;
              
              typeshare lib/rain.interpreter/crates/eval/src/trace.rs crates/common/src/fuzz/mod.rs crates/settings/src/config_source.rs crates/settings/src/config.rs crates/settings/src/plot_source.rs crates/settings/src/chart.rs crates/settings/src/deployer.rs crates/settings/src/network.rs crates/settings/src/order.rs crates/settings/src/orderbook.rs crates/settings/src/scenario.rs crates/settings/src/blocks.rs crates/settings/src/token.rs crates/settings/src/deployment.rs crates/settings/src/gui.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/config.ts;

              typeshare crates/common/src/dotrain_order/mod.rs lib/rain.interpreter/lib/rain.metadata/crates/cli/src/meta/types/authoring/v2.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/authoringMeta.ts;
              typeshare crates/quote/src/order_quotes.rs crates/quote/src/quote.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/orderQuote.ts;
              typeshare tauri-app/src-tauri/src/toast.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/toast.ts;
              typeshare tauri-app/src-tauri/src/transaction_status.rs --lang=typescript --output-file=tauri-app/src/lib/typeshare/transactionStatus.ts;

              # Fix linting of generated types
              cd tauri-app && npm i && npm run lint
            '';
            additionalBuildInputs = [
              pkgs.wasm-bindgen-cli
              rainix.rust-toolchain.${system}
              rainix.rust-build-inputs.${system}
            ];
          };

          ob-ui-components-prelude = rainix.mkTask.${system} {
            name = "ob-ui-components-prelude";
            body = ''
              set -euxo pipefail

              # Generate Typescript types from rust types
              mkdir -p packages/ui-components/src/lib/typeshare

              export CARGO_HOME=$(mktemp -d)
              cargo install --git https://github.com/tomjw64/typeshare --rev 556b44aafd5304eedf17206800f69834e3820b7c
              export PATH=$PATH:$CARGO_HOME/bin

              typeshare crates/subgraph/src/types/common.rs crates/subgraph/src/types/order.rs crates/subgraph/src/types/vault.rs crates/subgraph/src/types/order_trade.rs crates/common/src/types/order_detail_extended.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/subgraphTypes.ts;

              typeshare crates/settings/src/parse.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/appSettings.ts;
              
              typeshare lib/rain.interpreter/crates/eval/src/trace.rs crates/common/src/fuzz/mod.rs crates/settings/src/config_source.rs crates/settings/src/config.rs crates/settings/src/plot_source.rs crates/settings/src/chart.rs crates/settings/src/deployer.rs crates/settings/src/network.rs crates/settings/src/order.rs crates/settings/src/orderbook.rs crates/settings/src/scenario.rs crates/settings/src/blocks.rs crates/settings/src/token.rs crates/settings/src/deployment.rs crates/settings/src/gui.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/config.ts;

              typeshare crates/common/src/dotrain_order/mod.rs lib/rain.interpreter/lib/rain.metadata/crates/cli/src/meta/types/authoring/v2.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/authoringMeta.ts;
              typeshare crates/quote/src/order_quotes.rs crates/quote/src/quote.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/orderQuote.ts;
              typeshare tauri-app/src-tauri/src/toast.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/toast.ts;
              typeshare tauri-app/src-tauri/src/transaction_status.rs --lang=typescript --output-file=packages/ui-components/src/lib/typeshare/transactionStatus.ts;

              # Fix linting of generated types
              cd packages/ui-components && npm i && npm run lint
            '';
            additionalBuildInputs = [
              pkgs.wasm-bindgen-cli
              rainix.rust-toolchain.${system}
              rainix.rust-build-inputs.${system}
            ];
          };

          ob-tauri-unit-test =  rainix.mkTask.${system} {
            name = "ob-tauri-unit-test";
            body = ''
              set -euxo pipefail

              cd tauri-app && npm i && npm run test
            '';
          };

          ob-tauri-before-release = rainix.mkTask.${system} {
            name = "ob-tauri-before-release";
            body = ''
              # Idempotently, create new 'release' on sentry for the current commit
              sentry-cli releases new -p ''${SENTRY_PROJECT} ''${COMMIT_SHA}
              sentry-cli releases set-commits --auto ''${COMMIT_SHA}

              # Overwrite env variables with release values
              echo SENTRY_AUTH_TOKEN=''${SENTRY_AUTH_TOKEN} >> .env
              echo SENTRY_ORG=''${SENTRY_ORG} >> .env
              echo SENTRY_PROJECT=''${SENTRY_PROJECT} >> .env
              echo VITE_SENTRY_RELEASE=''${COMMIT_SHA} >> .env
              echo VITE_SENTRY_ENVIRONMENT=release >> .env
              echo VITE_SENTRY_FORCE_DISABLED=false >> .env
              echo VITE_SENTRY_DSN=''${SENTRY_DSN} >> .env
              echo COMMIT_SHA=''${COMMIT_SHA} >> .env
              echo VITE_WALLETCONNECT_PROJECT_ID=''${VITE_WALLETCONNECT_PROJECT_ID} >> .env
            '';
            additionalBuildInputs = [
              pkgs.sentry-cli
            ];
          };

          ob-tauri-before-build-ci = rainix.mkTask.${system} {
            name = "ob-tauri-before-build-ci";
            body = ''
              # Create env file with working defaults
              ENV_FILE=".env"
              ENV_EXAMPLE_FILE=".env.example"
              cp $ENV_EXAMPLE_FILE $ENV_FILE  

              # Update the existing WALLETCONNECT_PROJECT_ID line
              sed -i "s/^VITE_WALLETCONNECT_PROJECT_ID=.*/VITE_WALLETCONNECT_PROJECT_ID=''${WALLETCONNECT_PROJECT_ID}/" $ENV_FILE
            '';
          };

          ob-tauri-before-build = rainix.mkTask.${system} {
            name = "ob-tauri-before-build";
            body = ''
              set -euxo pipefail

              # Source .env file if it exists
              ENV_FILE=.env
              if [ -f "$ENV_FILE" ]; then
                  source $ENV_FILE
              fi

              # Exit if required env variables are not defined
              if [ -z "$VITE_WALLETCONNECT_PROJECT_ID" ]; then
                echo "Cancelling build: VITE_WALLETCONNECT_PROJECT_ID is not defined"
                exit 1
              fi

              if [ "$VITE_SENTRY_FORCE_DISABLED" != "true" ] &&
              (
                [ -z "$VITE_SENTRY_DSN" ] ||
                [ -z "$VITE_SENTRY_ENVIRONMENT" ] ||
                [ -z "$VITE_SENTRY_RELEASE" ]
              ); then
                echo "Cancelling build: EITHER env variable VITE_SENTRY_FORCE_DISABLED=true OR all env variables VITE_SENTRY_DSN, VITE_SENTRY_ENVIRONMENT and VITE_SENTRY_RELEASE must be defined"
                exit 1
              fi


              npm i && npm run build
              rm -rf lib
              mkdir -p lib

              if [ ${if pkgs.stdenv.isDarwin then "1" else "0" } -eq 1 ]; then
                cp ${pkgs.libiconv}/lib/libcharset.1.dylib lib/libcharset.1.dylib
                chmod +w lib/libcharset.1.dylib
                install_name_tool -id @executable_path/../Frameworks/libcharset.1.dylib lib/libcharset.1.dylib
                otool -L lib/libcharset.1.dylib

                cp ${pkgs.libiconv}/lib/libiconv.2.dylib lib/libiconv.2.dylib
                chmod +w lib/libiconv.2.dylib
                install_name_tool -id @executable_path/../Frameworks/libiconv.2.dylib lib/libiconv.2.dylib
                install_name_tool -change ${pkgs.libiconv}/lib/libcharset.1.dylib @executable_path/../Frameworks/libcharset.1.dylib lib/libiconv.2.dylib
                otool -L lib/libiconv.2.dylib

                cp ${pkgs.gettext}/lib/libintl.8.dylib lib/libintl.8.dylib
                chmod +w lib/libintl.8.dylib
                install_name_tool -id @executable_path/../Frameworks/libintl.8.dylib lib/libintl.8.dylib
                install_name_tool -change ${pkgs.libiconv}/lib/libiconv.2.dylib @executable_path/../Frameworks/libiconv.2.dylib lib/libintl.8.dylib
                otool -L lib/libintl.8.dylib

                cp ${pkgs.libusb}/lib/libusb-1.0.0.dylib lib/libusb-1.0.0.dylib
                chmod +w lib/libusb-1.0.0.dylib
                install_name_tool -id @executable_path/../Frameworks/libusb-1.0.0.dylib lib/libusb-1.0.0.dylib
                otool -L lib/libusb-1.0.0.dylib
              fi
            '';
          };

          ob-tauri-before-bundle = rainix.mkTask.${system} {
            name = "ob-tauri-before-bundle";
            body = ''
              set -euxo pipefail

              ls src-tauri/target/release

              if [ ${if pkgs.stdenv.isDarwin then "1" else "0" } -eq 1 ]; then
                install_name_tool -change ${pkgs.libiconv}/lib/libiconv.2.dylib @executable_path/../Frameworks/libiconv.2.dylib src-tauri/target/release/Raindex
                install_name_tool -change ${pkgs.gettext}/lib/libintl.8.dylib @executable_path/../Frameworks/libintl.8.dylib src-tauri/target/release/Raindex
                install_name_tool -change ${pkgs.libusb}/lib/libusb-1.0.0.dylib @executable_path/../Frameworks/libusb-1.0.0.dylib src-tauri/target/release/Raindex

                otool -L src-tauri/target/release/Raindex
                grep_exit_code=0
                otool -L src-tauri/target/release/Raindex | grep -q /nix/store || grep_exit_code=$?
                if [ $grep_exit_code -eq 0 ]; then
                  exit 1
                fi
              fi
            '';
          };

          rainix-wasm-artifacts = rainix.mkTask.${system} {
            name = "rainix-wasm-artifacts";
            body = ''
              set -euxo pipefail

              cargo build -r --target wasm32-unknown-unknown --lib --workspace --exclude rain_orderbook_cli --exclude rain_orderbook_integration_tests
            '';
          };

          rainix-wasm-test = rainix.mkTask.${system} {
            name = "rainix-wasm-test";
            body = ''
              set -euxo pipefail

              CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner' cargo test --target wasm32-unknown-unknown --lib -p rain_orderbook_quote -p rain_orderbook_bindings
            '';
          };

          js-install = rainix.mkTask.${system} {
            name = "js-install";
            body = ''
              set -euxo pipefail
              cd packages/orderbook
              npm install --no-check
            '';
          };

          build-js-bindings = rainix.mkTask.${system} {
            name = "build-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/orderbook
              npm run build
            '';
          };

          test-js-bindings = rainix.mkTask.${system} {
            name = "test-js-bindings";
            body = ''
              set -euxo pipefail
              cd packages/orderbook
              npm install --no-check
              npm run build
              npm test
            '';
          };

        } // rainix.packages.${system};

        devShells.default = pkgs.mkShell {
          packages = [
            packages.raindex-prelude
            packages.ob-rs-test
            packages.rainix-wasm-artifacts
            packages.rainix-wasm-test
            packages.js-install
            packages.build-js-bindings
            packages.test-js-bindings
            rain.defaultPackage.${system}
            packages.ob-ui-components-prelude
          ];

          shellHook = rainix.devShells.${system}.default.shellHook;
          buildInputs = rainix.devShells.${system}.default.buildInputs;
          nativeBuildInputs = rainix.devShells.${system}.default.nativeBuildInputs;
        };
        devShells.tauri-shell = pkgs.mkShell {
          packages = [
            packages.raindex-prelude
            packages.ob-tauri-prelude
            packages.ob-ui-components-prelude
            packages.ob-tauri-unit-test
            packages.ob-tauri-before-build-ci
            packages.ob-tauri-before-build
            packages.ob-tauri-before-bundle
            packages.ob-tauri-before-release
            packages.tauri-rs-test
          ];
          shellHook = rainix.devShells.${system}.tauri-shell.shellHook;
          buildInputs = rainix.devShells.${system}.tauri-shell.buildInputs ++ [pkgs.clang-tools];
          nativeBuildInputs = rainix.devShells.${system}.tauri-shell.nativeBuildInputs;
        };
        devShells.webapp-shell = pkgs.mkShell {
          packages = with pkgs; [
              nodejs_20
          ];
        };

      }
    );

}
