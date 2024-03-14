#!/bin/bash

# Run commands in the current working directory
nix develop --command rainix-sol-prelude
nix develop --command rainix-rs-prelude

# Run commands in lib/rain.interpreter
cd lib/rain.interpreter
nix develop --command rainix-sol-prelude
nix develop --command rainix-rs-prelude
nix develop --command i9r-prelude
cd -

nix develop .#tauri-shell --command ob-tauri-prelude
nix develop .#tauri-shell --command ob-tauri-test

# Run commands in tauri-app
cd tauri-app
nix develop .#tauri-shell --command cargo build --verbose
cd -
