# rain.orderbook

## Setup for local development

### Environment Setup

1. Copy `.env.example` to `.env`
2. Copy `tauri-app/.env.example` to `tauri-app/.env` and fill out `VITE_WALLETCONNECT_PROJECT_ID` with a test project ID from [Reown (FKA WalletConnect)](https://cloud.reown.com/sign-in)

Then run the following to install dependencies and build the project:
```bash
./prep-all.sh
```

You may need to make the shell script executable:
```bash
chmod +x prep-all.sh
```

### Run Webapp for local development
```
cd packages/webapp && nix develop -c npm run dev
```

### Run Tauri App for local development
```
nix develop .#tauri-shell --command cargo tauri dev
```
