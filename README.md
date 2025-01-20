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

## Legal stuff

Everything is under DecentraLicense 1.0 (DCL-1.0) which can be found in `LICENSES/`.

This is basically `CAL-1.0` which is an open source license
https://opensource.org/license/cal-1-0

The non-legal summary of DCL-1.0 is that the source is open, as expected, but
also user data in the systems that this code runs on must also be made available
to those users as relevant, and that private keys remain private.

Roughly it's "not your keys, not your coins" aware, as close as we could get in
legalese.

This is the default situation on permissionless blockchains, so shouldn't require
any additional effort by dev-users to adhere to the license terms.

This repo is REUSE 3.2 compliant https://reuse.software/spec-3.2/ and compatible
with `reuse` tooling (also available in the nix shell here).

```
nix develop -c rainix-sol-legal
```

## Contributions

Contributions are welcome **under the same license** as above.

Contributors agree and warrant that their contributions are compliant.