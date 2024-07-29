import { Bytes } from "@graphprotocol/graph-ts";
import { MetaV1 } from "../generated/OrderBook/OrderBook";
import { Order } from "../generated/schema";
import { makeOrderId } from "./order";
import { createOrderbookEntity } from "./orderbook";

export function handleMeta(event: MetaV1): void {
  createOrderbookEntity(event);

  let uint8Array = changetype<Uint8Array>(event.params.subject);
  // reverse it and remove any leading zeros
  uint8Array.reverse();
  while (uint8Array.length > 0 && uint8Array[0] == 0) {
    uint8Array = uint8Array.slice(1);
  }

  const byteArray = changetype<Bytes>(uint8Array);

  // The order should already exist by the time the MetaV1 event is handled
  let order = Order.load(makeOrderId(event.address, byteArray));
  if (order != null) {
    order.meta = event.params.meta;
    order.save();
  }
}
