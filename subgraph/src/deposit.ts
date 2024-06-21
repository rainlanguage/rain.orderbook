import { Bytes } from "@graphprotocol/graph-ts";
import { Deposit as DepositEntity } from "../generated/schema";
import { createTransactionEntity } from "./transaction";
import { Deposit } from "../generated/OrderBook/OrderBook";

export function handleDeposit(event: Deposit): void {
  createDepositEntity(event);
}

export function createDepositEntity(event: Deposit): void {
  let deposit = new DepositEntity(
    event.transaction.hash.concat(
      Bytes.fromByteArray(Bytes.fromBigInt(event.logIndex))
    )
  );
  deposit.amount = event.params.amount;
  deposit.sender = event.params.sender;
  deposit.vaultId = event.params.vaultId;
  deposit.token = event.params.token;
  deposit.transaction = createTransactionEntity(event);
  deposit.save();
}
