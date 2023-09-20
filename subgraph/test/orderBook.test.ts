import assert from "assert";
import { FetchResult } from "apollo-fetch";
import { orderBook, subgraph } from "./0_initialization.test";
import { ethers } from "hardhat";
import { getEventArgs } from "./utils";
import { MetaV1Event } from "../typechain/contracts/orderbook/OrderBook";

describe("Orderbook entity", () => {
  it("should query the OrderBook entity", async () => {
    const { subject, meta } = (await getEventArgs(
      orderBook.deployTransaction,
      "MetaV1",
      orderBook
    )) as MetaV1Event["args"];

    const orderBookAddress = orderBook.address.toLowerCase();
    const deployerAddress = orderBook.deployTransaction.from.toLowerCase();
    const obSubject = ethers.utils.hexZeroPad(subject.toHexString(), 20);
    const metaV1_ID = ethers.utils.keccak256(meta);

    assert(orderBookAddress === obSubject.toLowerCase(), "wrong OB subject");

    const query = `{
      orderBook(id: "${orderBookAddress}"){
        id
        address
        deployer
        meta {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;
    const data = response.data.orderBook;

    assert.equal(data.id, orderBookAddress, "Wrong orderbook ID");
    assert.equal(data.address, orderBookAddress, "Wrong orderbook address");
    assert.equal(data.deployer, deployerAddress, "Wrong deployer address ID");
    assert.equal(data.meta.id, metaV1_ID, "Wrong meta ID");
  });
});
