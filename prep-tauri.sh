#!/bin/bash

# Run commands in the current working directory
nix develop -i --command rainix-sol-prelude
nix develop -i --command rainix-rs-prelude
nix develop -i --command raindex-prelude

# Run commands in lib/rain.interpreter
cd lib/rain.interpreter
nix develop -i --command rainix-sol-prelude
nix develop -i --command rainix-rs-prelude
nix develop -i --command i9r-prelude
cd -

# Run commands in lib/rain.metadata
cd lib/rain.metadata
nix develop -i --command rainix-sol-prelude
cd -

nix develop -i .#tauri-shell --command ob-tauri-prelude
nix develop -i .#tauri-shell --command ob-tauri-unit-test

# Run commands in tauri-app
cd tauri-app
nix develop -i .#tauri-shell --command cargo build --verbose
cd -
