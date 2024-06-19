import { Deposit } from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { createDepositEntity } from "./deposit";
import { handleVaultDeposit, handleVaultWithdraw } from "./vault";
import { createWithdrawalEntity } from "./withdraw";

export function handleDeposit(event: Deposit): void {
  handleVaultDeposit(event);
}

export function handleWithdraw(event: Withdraw): void {
  handleVaultWithdraw(event);
}
