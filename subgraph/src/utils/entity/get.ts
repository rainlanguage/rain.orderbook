import { Address, Bytes } from "@graphprotocol/graph-ts";
import { OrderBook, RainMetaV1 } from "../../../generated/schema";
import { getKeccak256FromBytes } from "@rainprotocol/subgraph-utils";

export function getOB(obAddress: Address): OrderBook {
  let orderBook = OrderBook.load(obAddress);
  if (!orderBook) {
    orderBook = new OrderBook(obAddress);
    orderBook.address = obAddress;
    orderBook.save();
  }
  return orderBook;
}

export function getRainMetaV1(meta: Bytes): RainMetaV1 {
  const metaV1_ID = getKeccak256FromBytes(meta);

  let metaV1 = RainMetaV1.load(metaV1_ID);

  if (!metaV1) {
    metaV1 = new RainMetaV1(metaV1_ID);
    metaV1.metaBytes = meta;
    metaV1.save();
  }

  return metaV1;
}
