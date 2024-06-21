import {
  AddOrderV2,
  Deposit,
  RemoveOrderV2,
} from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { handleVaultDeposit, handleVaultWithdraw } from "./vault";
import {
  handleAddOrder as _handleAddOrder,
  handleRemoveOrder as _handleRemoveOrder,
} from "./order";

export function handleDeposit(event: Deposit): void {
  handleVaultDeposit(event);
}

export function handleWithdraw(event: Withdraw): void {
  handleVaultWithdraw(event);
}

export function handleAddOrder(event: AddOrderV2): void {
  _handleAddOrder(event);
}

export function handleRemoveOrder(event: RemoveOrderV2): void {
  _handleRemoveOrder(event);
}
