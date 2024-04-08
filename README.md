# rain.orderbook

Docs at https://rainprotocol.github.io/foundry.template

## To deploy the subgraph (temporary until we set up a new one)

The subgraph name should be {network}-{first 4 bytes of orderbook address}

```bash
cd subgraph
nix shell -p nodejs
npm i

# stay inside the previous shell for the next steps
nix run .#init-setup

nix run .#rain_cli -- subgraph build --network mainnet --block {orderbook deploy block} --address {orderbook address}

# these steps are for when we're running our own graph-node - may be different for each cloud service so best to check their docs
npx graph create --node {graph node address}:8020 {subgraph name} --access-token {auth key}

npx graph deploy {subgraph name} --ipfs {graph node url}:5001 --node {graph node url}:8020 --deploy-key {auth key} --headers '{ "Authorization": "Bearer {auth key}" }'
```


## Use as template

```
forge init -t rainprotocol/foundry.template <projectname>
cd <projectname>
forge install foundry-rs/forge-std
```

Then update the readme, set the docs url and configure github pages on github repo settings.

For CI deployments, setup all the environment variables and define contracts to
deploy in the matrix.
