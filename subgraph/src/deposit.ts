import { BigInt } from "@graphprotocol/graph-ts";
import { Deposit as DepositEntity } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTransactionEntity } from "./transaction";
import { vaultEntityId } from "./vault";
import { Deposit } from "../generated/OrderBook/OrderBook";

//
export function createDepositEntity(
  event: Deposit,
  oldVaultBalance: BigInt
): void {
  let deposit = new DepositEntity(eventId(event));
  deposit.amount = event.params.amount;
  deposit.sender = event.params.sender;
  deposit.vault = vaultEntityId(event.params.vaultId, event.params.token);
  deposit.token = event.params.token;
  deposit.transaction = createTransactionEntity(event);
  deposit.oldVaultBalance = oldVaultBalance;
  deposit.newVaultBalance = oldVaultBalance.plus(event.params.amount);
  deposit.save();
}
