import { Deposit } from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { createDepositEntity } from "./deposit";
import { handleVaultDeposit, handleVaultWithdraw } from "./vault";
import { createWithdrawalEntity } from "./withdraw";

export function handleDeposit(event: Deposit): void {
  createDepositEntity(event);
  handleVaultDeposit(event);
}

export function handleWithdraw(event: Withdraw): void {
  createWithdrawalEntity(event);
  handleVaultWithdraw(event);
}
