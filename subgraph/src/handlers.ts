import {
  AddOrderV3,
  AfterClearV2,
  ClearV3,
  DepositV2,
  MetaV1_2,
  RemoveOrderV3,
  TakeOrderV3,
} from "../generated/OrderBook/OrderBook";
import { WithdrawV2 } from "../generated/OrderBook/OrderBook";
import { log } from "@graphprotocol/graph-ts";
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

export function handleDeposit(event: DepositV2): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleDeposit(event);
  } catch (e) {
    log.error("Error in handleDeposit: {}", [e.toString()]);
  }
}

export function handleWithdraw(event: WithdrawV2): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleWithdraw(event);
  } catch (e) {
    log.error("Error in handleWithdraw: {}", [e.toString()]);
  }
}

export function handleAddOrder(event: AddOrderV3): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleAddOrder(event);
  } catch (e) {
    log.error("Error in handleAddOrder: {}", [e.toString()]);
  }
}

export function handleRemoveOrder(event: RemoveOrderV3): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleRemoveOrder(event);
  } catch (e) {
    log.error("Error in handleRemoveOrder: {}", [e.toString()]);
  }
}

export function handleTakeOrder(event: TakeOrderV3): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleTakeOrder(event);
  } catch (e) {
    log.error("Error in handleTakeOrder: {}", [e.toString()]);
  }
}

export function handleMeta(event: MetaV1_2): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleMeta(event);
  } catch (e) {
    log.error("Error in handleMeta: {}", [e.toString()]);
  }
}

export function handleClear(event: ClearV3): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleClear(event);
  } catch (e) {
    log.error("Error in handleClear: {}", [e.toString()]);
  }
}

export function handleAfterClear(event: AfterClearV2): void {
  try {
    createTransactionEntity(event);
    createOrderbookEntity(event);
    _handleAfterClear(event);
  } catch (e) {
    log.error("Error in handleAfterClear: {}", [e.toString()]);
  }
}
