import { BigInt } from "@graphprotocol/graph-ts";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { Withdrawal } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTransactionEntity } from "./transaction";
import { vaultEntityId } from "./vault";

//
export function createWithdrawalEntity(
  event: Withdraw,
  oldVaultBalance: BigInt
): void {
  let withdraw = new Withdrawal(eventId(event));
  withdraw.amount = event.params.amount;
  withdraw.targetAmount = event.params.targetAmount;
  withdraw.sender = event.params.sender;
  withdraw.vault = vaultEntityId(event.params.vaultId, event.params.token);
  withdraw.token = event.params.token;
  withdraw.transaction = createTransactionEntity(event);
  withdraw.oldVaultBalance = oldVaultBalance;
  withdraw.newVaultBalance = oldVaultBalance.minus(event.params.amount);
  withdraw.save();
}
