import { ethereum } from "@graphprotocol/graph-ts";
import { Orderbook } from "../generated/schema";

export function createOrderbookEntity(event: ethereum.Event): void {
  let orderbook = Orderbook.load(event.address);
  if (!orderbook) {
    orderbook = new Orderbook(event.address);
    orderbook.save();
  }
}
