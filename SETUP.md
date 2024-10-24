# Setup Raindex on a New Chain

## Step 1: Deploy Metaboard and Metaboard Subgraph

**Note: If the desired chain already has the Metaboard contract and subgraph deployed, skip this step.**

- Navigate to the [Rain Metadata Repo](https://github.com/rainlanguage/rain.metadata).
- Run the `manual-sol-artifacts` GitHub workflow to deploy the Metaboard contract on the desired network.
- Add the deployed contract address to `subgraph/subgraph.yml`.
- Deploy the subgraph by triggering the manual deployment action on CI.
- Add the new Metaboard contract address to the rainlanguage org Github variables.

## Step 2: Deploy Contracts (Orderbook, Arbs, Dispair, Subparsers) and Orderbook Subgraph

- Navigate to the [Rain Interpreter Repo](https://github.com/rainlanguage/rain.interpreter).

  - Run the `manual-sol-artifacts` manual action to deploy the Interpreter contracts.

- Navigate to the [Rain Orderbook Repo](https://github.com/rainlanguage/rain.orderbook).

  - Run the `manual-sol-artifacts` manual action to deploy the Orderbook-related contracts.

- Add the deployed Orderbook contract to the `networks.json` file in the [Rain Orderbook Repo](https://github.com/rainlanguage/rain.orderbook/blob/main/subgraph/networks.json).
- Deploy the Orderbook subgraph by running the "Deploy subgraph" manual CI action for the desired network.

## Step 3: Set Up Arb Bot

- Clone the [Rain Arb Bot Repo](https://github.com/rainlanguage/arb-bot) and run it using Node.js (`node arb-bot.js`), or pull the latest Docker image available on Docker Hub under `rainprotocol/arb-bot` with the `master` tag.
- Configure the bot using CLI arguments or environment variables. Essential parameters include:

```
  MAX_RATIO=  
  SLEEP=  
  HYPERDX_API_KEY=  
  TRACER_SERVICE_NAME=  
  SUBGRAPH=  
  ARB_ADDRESS=  
  RPC_URL=  
  BOT_MIN_BALANCE=  
  BOT_WALLET_PRIVATEKEY=  \
```

  - Alternatively, instead of `BOT_WALLET_PRIVATEKEY`, set up multiple wallets:

```
    MNEMONIC=  
    WALLET_COUNT=  
    TOPUP_AMOUNT=
```

  - Fund the main wallet (index 0) with gas tokens. The main wallet will fund other wallets with `TOPUP_AMOUNT` and periodically sweep bounties back to itself.

- Refer to `README.md` in the repository for additional options and CLI arguments.
- Real-time bot OTEL logs/spans are forwarded to `HYPERDX_API_KEY` under `TRACER_SERVICE_NAME`.

## Step 4: Add Setup to Pubstrats

- Add the following information to the [H20 Pubstrats Repo](https://github.com/rainlanguage/rain.dex.pubstrats) in the `src/settings.yml` file:
  - Metaboard subgraph
  - ExpressionDeployer contract
  - Orderbook contract
  - Orderbook subgraph
  - Chain details (RPC URL, chain ID, etc.)
- Ensure each item is placed under its respective field in `settings.yml`.
