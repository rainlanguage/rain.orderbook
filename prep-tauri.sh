#!/bin/bash

# Run commands in the current working directory
nix develop -i --command rainix-sol-prelude
nix develop -i --command rainix-rs-prelude

# Run commands in lib/rain.interpreter
cd lib/rain.interpreter
nix develop -i --command rainix-sol-prelude
nix develop -i --command rainix-rs-prelude
nix develop -i --command i9r-prelude
cd -

nix develop .#tauri-shell --command ob-tauri-prelude
nix develop -i .#tauri-shell --command ob-tauri-test

# Run commands in tauri-app
cd tauri-app
nix develop -i .#tauri-shell --command cargo build --verbose
cd -
