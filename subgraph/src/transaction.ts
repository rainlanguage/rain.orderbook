import { ethereum } from "@graphprotocol/graph-ts";
import { Transaction } from "../generated/schema";

export function createTransactionEntity(event: ethereum.Event): string {
  let transaction = new Transaction(event.transaction.hash.toHex());
  transaction.blockNumber = event.block.number;
  transaction.timestamp = event.block.timestamp;
  transaction.from = event.transaction.from;
  transaction.save();
  return transaction.id;
}
