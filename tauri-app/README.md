# Tauri App

Tauri desktop app for managing orderbooks.

## Developing

To start a development server and run your tauri app in dev mode:

```bash
nix develop .#tauri-shell --command cargo tauri dev 
```

## Building

To create a production build of your app:

```bash
nix develop .#tauri-shell --command cargo tauri build 
```
