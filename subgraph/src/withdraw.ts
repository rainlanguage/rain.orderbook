import { BigInt } from "@graphprotocol/graph-ts";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { Withdrawal } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { handleVaultBalanceChange, vaultEntityId } from "./vault";

export function handleWithdraw(event: Withdraw): void {
  let oldVaultBalance: BigInt = handleVaultBalanceChange(
    event.address,
    event.params.vaultId,
    event.params.token,
    event.params.amount.neg(),
    event.params.sender
  );
  createWithdrawalEntity(event, oldVaultBalance);
}

export function createWithdrawalEntity(
  event: Withdraw,
  oldVaultBalance: BigInt
): void {
  let withdraw = new Withdrawal(eventId(event));
  withdraw.orderbook = event.address;
  withdraw.amount = event.params.amount.neg();
  withdraw.targetAmount = event.params.targetAmount;
  withdraw.sender = event.params.sender;
  withdraw.vault = vaultEntityId(
    event.address,
    event.params.sender,
    event.params.vaultId,
    event.params.token
  );
  withdraw.transaction = event.transaction.hash;
  withdraw.oldVaultBalance = oldVaultBalance;
  withdraw.newVaultBalance = oldVaultBalance.minus(event.params.amount);
  withdraw.timestamp = event.block.timestamp;
  withdraw.save();
}
