import { ethers } from "hardhat";
import {
  exec,
  fetchFile,
  fetchSubgraph,
  waitForGraphNode,
  waitForSubgraphToBeSynced,
  writeFile,
} from "./utils";
import { OrderBook } from "../typechain";
import assert from "assert";
import * as path from "path";
import { ApolloFetch } from "apollo-fetch";
import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import deploy1820 from "../utils/deploy/registry1820/deploy";
import { deployOrderBook } from "../utils/deploy/orderBook/deploy";
import { zeroAddress } from "../utils";

export let orderBook: OrderBook;
export let subgraph: ApolloFetch;
export let signers: SignerWithAddress[];

before(async () => {
  // Wait for the correct initialization of the graph node
  await waitForGraphNode();

  // Making available the Registry (EIP-1820) on local network
  signers = await ethers.getSigners();
  await deploy1820(signers[0]);

  orderBook = await deployOrderBook();

  const configPath = path.resolve(__dirname, "../config/localhost.json");

  assert(!(orderBook.address === zeroAddress), "OrderBook did not deploy");

  const config = JSON.parse(fetchFile(configPath));

  config.network = "localhost";
  config.orderbook = orderBook.address;
  config.blockNumber = orderBook.deployTransaction.blockNumber;

  writeFile(configPath, JSON.stringify(config, null, 2));

  // create subgraph instance
  exec("graph create --node http://localhost:8020/ test/test");
  // prepare subgraph manifest
  exec(
    "npx mustache config/localhost.json subgraph.template.yaml subgraph.yaml"
  );
  // deploy subgraph
  exec(
    "graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001 test/test  --version-label 1"
  );
  subgraph = fetchSubgraph("test/test");

  await waitForSubgraphToBeSynced(1000);
});
