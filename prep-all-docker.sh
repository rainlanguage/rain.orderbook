#!/bin/bash

# Docker-based version of prep-all.sh
# This version uses docker-nix.sh to run nix commands in Docker

# Set strict error handling
set -euxo pipefail

echo "Starting project setup using Docker for Nix..."

# Check if docker-nix.sh exists
if [ ! -f "./docker-nix.sh" ]; then
    echo "Error: docker-nix.sh not found. Please ensure it exists in the current directory."
    exit 1
fi

# Make sure docker-nix.sh is executable
chmod +x ./docker-nix.sh

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
  -k PUBLIC_WALLETCONNECT_PROJECT_ID
)

echo "Installing Forge dependencies..."
./docker-nix.sh develop -c forge install

echo "Setting up rain.math.float..."
./docker-nix.sh develop -i ${keep[@]} -c bash \
  -c '(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && rainix-sol-prelude)'
./docker-nix.sh develop -i ${keep[@]} -c bash \
  -c '(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && rainix-rs-prelude)'

echo "Setting up rain.interpreter..."
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-sol-prelude)'
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
(cd lib/rain.interpreter && ../../../docker-nix.sh develop -i ${keep[@]} -c bash -c i9r-prelude)

echo "Setting up rain.metadata..."
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-sol-prelude)'
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-rs-prelude)'

echo "Setting up main project dependencies..."
./docker-nix.sh develop -i ${keep[@]} -c rainix-sol-prelude
./docker-nix.sh develop -i ${keep[@]} -c rainix-rs-prelude
./docker-nix.sh develop -i ${keep[@]} -c raindex-prelude

echo "Setting up UI components..."
./docker-nix.sh develop -i ${keep[@]} .#tauri-shell -c ob-tauri-prelude
./docker-nix.sh develop -i ${keep[@]} .#tauri-shell -c ob-ui-components-prelude

echo "Building packages..."
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(npm run build -w @rainlanguage/orderbook)'
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(npm run build -w @rainlanguage/ui-components && npm run build -w @rainlanguage/webapp)'
./docker-nix.sh develop -i ${keep[@]} -c bash -c '(npm run build -w tauri-app)'

# Temporarily disable command echoing
set +x

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Print the completion message
printf "\033[0;32m" # Set text to green
printf "╔════════════════════════════════════════════════════════════════════════╗\n"
printf "║                            Setup Complete!                             ║\n"
printf "╠════════════════════════════════════════════════════════════════════════╣\n"
printf "║                          How to run the apps:                          ║\n"
printf "║                                                                        ║\n"
printf "║  To run webapp:     cd packages/webapp && ../../docker-nix.sh develop -c npm run dev   ║\n"
printf "║  To run tauri app:  ./docker-nix.sh develop .#tauri-shell -c cargo tauri dev       ║\n"
printf "║  To run cargo tests: ./docker-nix.sh develop -c cargo test --workspace       ║\n"
printf "╚════════════════════════════════════════════════════════════════════════╝\n"
printf "\033[0m" # Reset text color

# Re-enable command echoing
set -x