# rain.orderbook

## Setup for local development

## Environment Setup

1. Copy `.env.example` to `.env`
2. Copy `tauri-app/.env.example` to `tauri-app/.env` and fill in values

Then run the following to install dependencies and build the project:
```bash
./prep-all.sh
```

You may need to make the shell script executable:
```bash
chmod +x prep-all.sh
```

## Run Webapp for local development
```
nix develop -c npm run dev -w @rainlanguage/webapp
```

## Run Tauri App for local development
```
nix develop .#tauri-shell --command cargo tauri dev
```

## Use as template

Docs at https://rainprotocol.github.io/foundry.template

```
forge init -t rainprotocol/foundry.template <projectname>
cd <projectname>
forge install foundry-rs/forge-std
```

Then update the readme, set the docs url and configure github pages on github repo settings.

For CI deployments, setup all the environment variables and define contracts to
deploy in the matrix.
