#!/bin/bash

set -euxo pipefail

keep=(
  -k
  CI_DEPLOY_SEPOLIA_RPC_URL
  -k
  CI_FORK_SEPOLIA_DEPLOYER_ADDRESS
  -k
  CI_FORK_SEPOLIA_BLOCK_NUMBER
)

# Run commands in the current working directory
nix develop -i ${keep[@]} -c rainix-sol-prelude
nix develop -i ${keep[@]} -c rainix-rs-prelude
nix develop -i ${keep[@]} -c raindex-prelude

# Run commands in lib/rain.interpreter
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-sol-prelude)'
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
(cd lib/rain.interpreter && nix develop -i ${keep[@]} -c bash -c i9r-prelude)

# Run commands in lib/rain.metadata
nix develop -i ${keep[@]} -c bash -c '(cd lib/rain.metadata && rainix-sol-prelude)'

nix develop -i ${keep[@]} .#tauri-shell -c ob-tauri-prelude
nix develop -i ${keep[@]} .#tauri-shell -c ob-tauri-unit-test

# Run commands in tauri-app
nix develop -i ${keep[@]} .#tauri-shell -c bash -c '(cd tauri-app && cargo build --verbose)'
