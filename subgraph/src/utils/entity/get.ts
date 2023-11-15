import { Address, Bytes } from "@graphprotocol/graph-ts";
import { OrderBook, RainMetaV1 } from "../../../generated/schema";
import { getKeccak256FromBytes } from "../../utils";

export function getOB(obAddress_: Address): OrderBook {
  let orderBook = OrderBook.load(obAddress_);
  if (!orderBook) {
    orderBook = new OrderBook(obAddress_);
    orderBook.address = obAddress_;
    orderBook.save();
  }
  return orderBook;
}

export function getRainMetaV1(meta_: Bytes): RainMetaV1 {
  const metaV1_ID = getKeccak256FromBytes(meta_);

  let metaV1 = RainMetaV1.load(metaV1_ID);

  if (!metaV1) {
    metaV1 = new RainMetaV1(metaV1_ID);
    metaV1.metaBytes = meta_;
    metaV1.content = [];
    metaV1.save();
  }

  return metaV1;
}
