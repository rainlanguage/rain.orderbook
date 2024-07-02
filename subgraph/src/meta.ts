import { Bytes } from "@graphprotocol/graph-ts";
import { MetaV1 } from "../generated/OrderBook/OrderBook";
import { Order } from "../generated/schema";
import { makeOrderId } from "./order";

export function handleMeta(event: MetaV1): void {
  // The order should already exist by the time the MetaV1 event is handled
  let order = Order.load(
    makeOrderId(
      event.address,
      Bytes.fromByteArray(Bytes.fromBigInt(event.params.subject))
    )
  );
  if (order != null) {
    order.meta = event.params.meta;
    order.save();
  }
}
