import { BigInt } from "@graphprotocol/graph-ts";
import { Deposit as DepositEntity } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTransactionEntity } from "./transaction";
import { handleVaultBalanceChange, vaultEntityId } from "./vault";
import { Deposit } from "../generated/OrderBook/OrderBook";
import { createOrderbookEntity } from "./orderbook";

export function handleDeposit(event: Deposit): void {
  createOrderbookEntity(event);
  let oldVaultBalance: BigInt = handleVaultBalanceChange(
    event.address,
    event.params.vaultId,
    event.params.token,
    event.params.amount,
    event.params.sender
  );
  createDepositEntity(event, oldVaultBalance);
}
export function createDepositEntity(
  event: Deposit,
  oldVaultBalance: BigInt
): void {
  let deposit = new DepositEntity(eventId(event));
  deposit.orderbook = event.address;
  deposit.amount = event.params.amount;
  deposit.sender = event.params.sender;
  deposit.vault = vaultEntityId(
    event.address,
    event.params.sender,
    event.params.vaultId,
    event.params.token
  );
  deposit.transaction = createTransactionEntity(event);
  deposit.oldVaultBalance = oldVaultBalance;
  deposit.newVaultBalance = oldVaultBalance.plus(event.params.amount);
  deposit.timestamp = event.block.timestamp;
  deposit.save();
}
