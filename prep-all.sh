#!/bin/bash

# Set strict error handling
set -euxo pipefail

echo "Starting project setup..."

# Environment variables that need to be set (commented out as reference)
# export CI_DEPLOY_SEPOLIA_RPC_URL=""
# export CI_FORK_SEPOLIA_DEPLOYER_ADDRESS=""
# export CI_FORK_SEPOLIA_BLOCK_NUMBER=""
# export CI_DEPLOY_POLYGON_RPC_URL=""
# export CI_SEPOLIA_METABOARD_URL=""
# export RPC_URL_ETHEREUM_FORK=""
# export COMMIT_SHA=""

# Keep environment variables when using nix-develop
keep=(
  -k CI_DEPLOY_SEPOLIA_RPC_URL
  -k CI_FORK_SEPOLIA_DEPLOYER_ADDRESS
  -k CI_FORK_SEPOLIA_BLOCK_NUMBER
  -k CI_DEPLOY_POLYGON_RPC_URL
  -k CI_SEPOLIA_METABOARD_URL
  -k RPC_URL_ETHEREUM_FORK
  -k COMMIT_SHA
)

echo "Installing Forge dependencies..."
nix develop -c forge install

echo "Setting up rain.interpreter..."
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-sol-prelude)'
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
(cd lib/rain.interpreter && nix develop -i ${keep[@]} -c bash -c i9r-prelude)

echo "Setting up rain.metadata..."
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-sol-prelude)'
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-rs-prelude)'

echo "Setting up main project dependencies..."
nix develop -i ${keep[@]} -c rainix-sol-prelude
nix develop -i ${keep[@]} -c rainix-rs-prelude
nix develop -i ${keep[@]} -c raindex-prelude

echo "Setting up UI components..."
nix develop -i ${keep[@]} .#tauri-shell -c ob-tauri-prelude
nix develop -i ${keep[@]} .#tauri-shell -c ob-ui-components-prelude

echo "Building packages..."
nix develop -i ${keep[@]} -c npm run build -w @rainlanguage/orderbook
nix develop -i ${keep[@]} -c bash -c '(npm run build -w @rainlanguage/ui-components && npm run build -w @rainlanguage/webapp)'

echo -e "\n┌────────────────────────────────────────────────────────────────────────┐"
echo -e "│\033[32m                            Setup Complete!                             \033[0m│"
echo -e "├────────────────────────────────────────────────────────────────────────┤"
echo -e "│\033[32m                          How to run the apps:                          \033[0m│"
echo -e "│\033[32m                                                                        \033[0m│"
echo -e "│\033[32m  To run webapp:     nix develop -c 'cd packages/webapp && npm run dev' \033[0m│"
echo -e "│\033[32m  To run tauri app:  nix develop .#tauri-shell -c cargo tauri dev       \033[0m│"
echo -e "└────────────────────────────────────────────────────────────────────────┘\n"