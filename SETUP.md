# Setup Raindex on a new chain

### Deploying metaboard and metaboard subgraph
If the desired chain already has metaboard contract and subgraph deployed you can skip this stage.
Head to [Rain Metadata Repo](https://github.com/rainlanguage/rain.metadata) and by through running the `manual-sol-artifacts` github workflow deploy the metaboard contract on the desired network, next add the deployed contract address to `subgraph/subgraph.yml` and deploy the subgraph using goldsky cli and any other graph cli app, you need to run 2 commands before deploying to build the subgraph:
```sh
npx graph codegen
npx graph build
```
and run the following command if you are using goldsky as your subgraph infra:
```sh
cd subgraph && goldsky subgraph deploy 'subgraph-name/subgraph-version'
```

### Deploying all contracts including ob, arbs, dispair and subparsers and orderbook subgraph
Head to [Rain Interpreter Repo](https://github.com/rainlanguage/rain.interpreter) and by through running the `manual-sol-artifacts` github workflow deploy the rain interpreter contract, note that you would need to add the metaboard subgraph endpoint as an env.

Next head to [Rain Orderbook Repo](https://github.com/rainlanguage/rain.orderbook) and repeat the same process there, to deploy the orderbook related contracts (note that you would need to add the metaboard subgraph endpoint as an env.). Now that orderbook contract is deployed add its address to `subgraph/subgraph.yml` and deploy the orderbook subgraph using goldsky cli or any other graph cli app, you need to run 2 commands before deploying to build the subgraph:
```sh
npx graph codegen
npx graph build
```
and run the following command if you are using goldsky as your subgraph infra:
```sh
cd subgraph && goldsky subgraph deploy 'subgraph-name/subgraph-version'
```

### Setup bot
You can run the bot either by cloning the [Rain Arb Bot Repo](https://github.com/rainlanguage/arb-bot) and running it using nodejs (ie `node arb-bot.js`) or you can pull the latest docker image available on dockerhub under `rainprotocol/arb-bot` with `master` tag.
You need to setup the configuration for the bot either through cli args or through env variables, the main args that are essential for bot operation are:
```sh
MAX_RATIO=
SLEEP=
HYPERDX_API_KEY=
TRACER_SERVICE_NAME=
SUBGRAPH=
ARB_ADDRESS=
RPC_URL=
BOT_MIN_BALANCE=
BOT_WALLET_PRIVATEKEY=
```
alternatively instead of `BOT_WALLET_PRIVATEKEY` you can set the following to have the bot run multiple wallets in rotation instead of using only 1:
```sh
MNEMONIC=
WALLET_COUNT=
TOPUP_AMOUNT=
```
When using this setup, you only need to fund the main wallet (0 derivation index) of the specified mnemonic phrase with gas token and it will fund the other wallets with the amount specified at `TOPUP_AMOUNT` and sweep the bounties back to the main wallet periodically

These args are set through env vars, their cli arg alternative are documentated on the `README.md` in the repo if you are directly running the bot through cli.
Other bot options are also documented on `README.md`, which for most usecases should not be necessary.

The realtime bot otel logs/spans will be forwarded to the specified `HYPERDX_API_KEY` under the specified `TRACER_SERVICE_NAME`.

### Add the new setup to Pubstrats
Lastly, add Metaboard subgraph, ExpressionDeployer contract and Orderbook contract and subgraph with the chain's details (rpc url, chaid id, etc) to [H20 Pubstrats Repo](https://github.com/rainlanguage/rain.dex.pubstrats) to `src/settings.yml` file, each item should go under its respective field which is self explanatory.
