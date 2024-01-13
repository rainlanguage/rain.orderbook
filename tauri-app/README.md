# Tauri App

Tauri desktop app for managing orderbooks.

## Developing

To start in dev mode, run the following command from the repository root:

```bash
nix develop .#tauri-shell --command cargo tauri dev
```

## Building

To create a production build, run the following command from the repository root:

```bash
nix develop .#tauri-shell --command cargo tauri build
```
