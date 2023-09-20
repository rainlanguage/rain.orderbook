import { execSync } from "child_process";
import fs from "fs";
import { Contract, ContractTransaction } from "ethers";
import { Result } from "ethers/lib/utils";
import { ethers } from "hardhat";
import * as path from "path";
import { ApolloFetch, createApolloFetch } from "apollo-fetch";
import { OBMultiTx, OrderBook } from "../typechain";

export const META_MAGIC_NUMBER_V1 = BigInt(0xff0a89c674ee7874n);
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const writeFile = (_path: string, file: any): void => {
  try {
    fs.writeFileSync(_path, file);
  } catch (error) {
    console.log(error);
  }
};

export const fetchFile = (_path: string): string => {
  try {
    return fs.readFileSync(_path).toString();
  } catch (error) {
    console.log(error);
    return "";
  }
};

export const getEventArgs = async (
  tx: ContractTransaction,
  eventName: string,
  contract: Contract,
  contractAddressOverride: string = null
): Promise<Result> => {
  const eventObj = (await tx.wait()).events.find(
    (x) =>
      x.topics[0] === contract.filters[eventName]().topics[0] &&
      x.address === (contractAddressOverride || contract.address)
  );

  if (!eventObj) {
    throw new Error(`Could not find event with name ${eventName}`);
  }

  // Return all events indexed and not indexed
  return contract.interface.decodeEventLog(
    eventName,
    eventObj.data,
    eventObj.topics
  );
};

export const appendRainMetaDoc = (data: string | string[]) => {
  const startMeta = "0x" + MAGIC_NUMBERS.RAIN_META_DOCUMENT.toString(16);

  if (Array.isArray(data)) {
    return startMeta + data.join("");
  } else {
    return startMeta + data;
  }
};

export const MAGIC_NUMBERS = {
  /**
   * Prefixes every rain meta document
   */
  RAIN_META_DOCUMENT: BigInt(0xff0a89c674ee7874n),
  /**
   * Solidity ABIv2
   */
  SOLIDITY_ABIV2: BigInt(0xffe5ffb4a3ff2cden),
  /**
   * Ops meta v1
   */
  OPS_META_V1: BigInt(0xffe5282f43e495b4n),
  /**
   * Contract meta v1
   */
  CONTRACT_META_V1: BigInt(0xffc21bbf86cc199bn),
};

/**
 * Execute Child Processes
 * @param cmd Command to execute
 * @returns The command ran it
 */
export const exec = (cmd: string): string | Buffer => {
  const srcDir = path.join(__dirname, "..");
  try {
    return execSync(cmd, { cwd: srcDir, stdio: "inherit" });
  } catch (e) {
    console.log(e);
    throw new Error(`Failed to run command \`${cmd}\``);
  }
};

// Subgraph Management
export const fetchSubgraphs = process.env.RPC_URL
  ? createApolloFetch({
      uri: `${process.env.RPC_URL}:8030/graphql`,
    })
  : createApolloFetch({
      uri: `http://localhost:8030/graphql`,
    });

/**
 * Connect to an existing subgraph deployed in localhost
 * @param subgraphName Name of the subgraph
 * @returns connection to subgraph
 */
export const fetchSubgraph = (subgraphName: string): ApolloFetch => {
  return process.env.RPC_URL
    ? createApolloFetch({
        uri: `${process.env.RPC_URL}:8000/subgraphs/name/${subgraphName}`,
      })
    : createApolloFetch({
        uri: `http://localhost:8000/subgraphs/name/${subgraphName}`,
      });
};

/**
 * Create a promise to wait a determinated `ms`
 * @param ms Amount of time to wait in miliseconds
 */
export function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Interfaces
interface SyncedSubgraphType {
  synced: boolean;
}

export const waitForSubgraphToBeSynced = async (
  wait = 0,
  timeDelay = 1,
  seconds = 60,
  subgraphName = "test/test"
): Promise<SyncedSubgraphType> => {
  if (wait > 0) {
    await delay(wait);
  }
  /**
   * Waiting for 60s by default
   * Does not care about waiting the 60s -  the function already try to handle if does not receive
   * a response. If the subgraph need to wait for a big number of blocks, would be good increse
   * the seconds to wait by sync.
   */
  const deadline = Date.now() + seconds * 1000;
  const currentBlock = await ethers.provider.getBlockNumber();

  const resp = new Promise<SyncedSubgraphType>((resolve, reject) => {
    // Function to check if the subgraph is synced asking to the GraphNode
    const checkSubgraphSynced = async () => {
      try {
        const result = await fetchSubgraphs({
          query: `
              {
                indexingStatusForCurrentVersion(subgraphName: "${subgraphName}") {
                  synced
                  health
                  fatalError{
                    message
                    handler
                  }
                  chains {
                    chainHeadBlock {
                      number
                    }
                    latestBlock {
                      number
                    }
                  }
                } 
              } 
            `,
        });
        const data = result.data.indexingStatusForCurrentVersion;
        if (
          data.synced === true &&
          data.chains[0].latestBlock.number == currentBlock
        ) {
          resolve({ synced: true });
        } else if (data.health === "failed") {
          reject(new Error(`Subgraph fatalError - ${data.fatalError.message}`));
        } else {
          throw new Error(`subgraph is not sync`);
        }
      } catch (e) {
        const message = e instanceof Error ? e.message : "Unknown Error";
        if (message.includes("connect ECONNREFUSED")) {
          reject(new Error(`Unable to connect to Subgraph node: ${message}`));
        }

        if (message == "Unknown Error") {
          reject(new Error(`${message} - ${e}`));
        }

        if (!currentBlock) {
          reject(new Error(`current block is undefined`));
        }

        if (e instanceof TypeError) {
          reject(
            new Error(
              `${e.message} - Check that the subgraphName provided is correct.`
            )
          );
        }

        if (Date.now() > deadline) {
          reject(new Error(`Timed out waiting for the subgraph to sync`));
        } else {
          setTimeout(checkSubgraphSynced, timeDelay * 1000);
        }
      }
    };

    checkSubgraphSynced();
  });

  return resp;
};

function sleep(milliseconds: number) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export const waitForGraphNode = async (): Promise<void> => {
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      //@ts-expect-error fetch is already available on node
      const response = await fetch("http://localhost:8030");
      if (response.status === 200) {
        break;
      }
    } catch (error) {
      console.log("graph node not ready");
      await sleep(100);
    }
  }
};

/**
 * Get the block and timestamp of a specific transaction
 * @param tx Transaction that will be use to get the block and timestamp
 * @returns The block and timestamp of the transaction
 */
export const getTxTimeblock = async (
  tx: ContractTransaction
): Promise<[number, number]> => {
  const block = tx.blockNumber;
  if (block == undefined) return [0, 0];
  const timestamp = (await ethers.provider.getBlock(block)).timestamp;
  return [block, timestamp];
};

export const deployOBMultiTx = async (orderBook: OrderBook) => {
  const contractFactory = await ethers.getContractFactory("OBMultiTx");
  const contract = await contractFactory.deploy(orderBook.address);

  await contract.deployed();

  return contract as OBMultiTx;
};
