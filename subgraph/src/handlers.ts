import {
  AddOrderV2,
  AfterClear,
  ClearV2,
  Deposit,
  MetaV1_2,
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
import {
  handleClear as _handleClear,
  handleAfterClear as _handleAfterClear,
} from "./clear";
import { createTransactionEntity } from "./transaction";
import { createOrderbookEntity } from "./orderbook";

export function handleDeposit(event: Deposit): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleDeposit(event);
}

export function handleWithdraw(event: Withdraw): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleWithdraw(event);
}

export function handleAddOrder(event: AddOrderV2): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleAddOrder(event);
}

export function handleRemoveOrder(event: RemoveOrderV2): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleRemoveOrder(event);
}

export function handleTakeOrder(event: TakeOrderV2): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleTakeOrder(event);
}

export function handleMeta(event: MetaV1_2): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleMeta(event);
}

export function handleClear(event: ClearV2): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleClear(event);
}

export function handleAfterClear(event: AfterClear): void {
  createTransactionEntity(event);
  createOrderbookEntity(event);
  _handleAfterClear(event);
}
