#!/bin/bash

# Run commands in the current working directory
nix develop --command rainix-sol-prelude
nix develop --command rainix-rs-prelude

# Run commands in lib/rain.interpreter
pushd lib/rain.interpreter
nix develop --command rainix-sol-prelude
nix develop --command rainix-rs-prelude
nix develop --command i9r-prelude
popd

# Run commands in tauri-app
pushd tauri-app
nix develop .#tauri-shell --command cargo build --verbose
popd
nix develop .#tauri-shell --command ob-tauri-prelude
nix develop .#tauri-shell --command ob-tauri-test
# Use tauri dev instead of tauri build
pushd tauri-app
nix develop .#tauri-shell --command cargo tauri dev --verbose
popd
