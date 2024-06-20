import { AddOrderV2 } from "../generated/OrderBook/OrderBook";
import { AddOrder, Order } from "../generated/schema";

export function handleAddOrder(event: AddOrderV2): void {
  let order = new Order(event.params.orderHash);
}
