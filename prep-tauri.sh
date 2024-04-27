#!/bin/bash

# Run commands in the current working directory
nix develop -i -c rainix-sol-prelude
nix develop -i -c rainix-rs-prelude
nix develop -i -c raindex-prelude

# Run commands in lib/rain.interpreter
nix develop -i -c bash -c '(cd lib/rain.interpreter && rainix-sol-prelude)'
nix develop -i -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
nix develop -i -c bash -c '(cd lib/rain.interpreter && i9r-prelude)'

# Run commands in lib/rain.metadata
nix develop -i -c bash -c '(cd lib/rain.metadata && rainix-sol-prelude)'

nix develop -i .#tauri-shell -c ob-tauri-prelude
nix develop -i .#tauri-shell -c ob-tauri-unit-test

# Run commands in tauri-app
nix develop -i .#tauri-shell -c bash -c '(cd tauri-app && cargo build --verbose)'
