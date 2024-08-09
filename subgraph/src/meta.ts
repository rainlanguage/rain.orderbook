import { MetaV1_2 } from "../generated/OrderBook/OrderBook";
import { Order } from "../generated/schema";
import { makeOrderId } from "./order";
import { createOrderbookEntity } from "./orderbook";

export function handleMeta(event: MetaV1_2): void {
  createOrderbookEntity(event);

  // The order should already exist by the time the MetaV1 event is handled
  let order = Order.load(
    makeOrderId(
      event.address,
      event.params.subject
    )
  );
  if (order != null) {
    order.meta = event.params.meta;
    order.save();
  }
}
