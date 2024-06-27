import {
  AddOrderV2,
  Deposit,
  MetaV1,
  RemoveOrderV2,
  TakeOrderV2,
} from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { handleDeposit as _handleDeposit } from "./deposit";
import { handleWithdraw as _handleWithdraw } from "./withdraw";
import {
  handleAddOrder as _handleAddOrder,
  handleRemoveOrder as _handleRemoveOrder,
} from "./order";
import { handleMeta as _handleMeta } from "./meta";
import { handleTakeOrder as _handleTakeOrder } from "./takeorder";

export function handleDeposit(event: Deposit): void {
  _handleDeposit(event);
}

export function handleWithdraw(event: Withdraw): void {
  _handleWithdraw(event);
}

export function handleAddOrder(event: AddOrderV2): void {
  _handleAddOrder(event);
}

export function handleRemoveOrder(event: RemoveOrderV2): void {
  _handleRemoveOrder(event);
}

export function handleTakeOrder(event: TakeOrderV2): void {
  _handleTakeOrder(event);
}

export function handleMeta(event: MetaV1): void {
  _handleMeta(event);
}
