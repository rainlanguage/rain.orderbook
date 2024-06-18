import { Deposit } from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { createDepositEntity } from "./deposit";
import { createWithdrawalEntity } from "./withdraw";

export function handleDeposit(event: Deposit): void {
  createDepositEntity(event);
}

export function handleWithdraw(event: Withdraw): void {
  createWithdrawalEntity(event);
}
