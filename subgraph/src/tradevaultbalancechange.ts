import { BigInt, Bytes, ethereum } from "@graphprotocol/graph-ts";
import { TradeVaultBalanceChange } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { makeTradeId } from "./trade";

export function tradeVaultBalanceChangeId(
  event: ethereum.Event,
  vaultEntityId: Bytes
): Bytes {
  return vaultEntityId.concat(eventId(event));
}

export function createTradeVaultBalanceChangeEntity(
  event: ethereum.Event,
  orderHash: Bytes,
  vaultEntityId: Bytes,
  oldVaultBalance: BigInt,
  amount: BigInt
): TradeVaultBalanceChange {
  let tradeVaultBalanceChange = new TradeVaultBalanceChange(
    tradeVaultBalanceChangeId(event, vaultEntityId)
  );
  tradeVaultBalanceChange.amount = amount;
  tradeVaultBalanceChange.oldVaultBalance = oldVaultBalance;
  tradeVaultBalanceChange.newVaultBalance = oldVaultBalance.plus(amount);
  tradeVaultBalanceChange.vault = vaultEntityId;
  tradeVaultBalanceChange.trade = makeTradeId(event, orderHash);
  tradeVaultBalanceChange.timestamp = event.block.timestamp;
  tradeVaultBalanceChange.transaction = event.transaction.hash;
  tradeVaultBalanceChange.save();
  return tradeVaultBalanceChange;
}
