import { BigInt, Bytes, ethereum } from "@graphprotocol/graph-ts";
import { TradeVaultBalanceChange } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { VaultBalanceChangeType } from "./vault";

export function tradeVaultBalanceChangeId(
  event: ethereum.Event,
  vaultEntityId: Bytes
): Bytes {
  return vaultEntityId.concat(eventId(event));
}

export function createTradeVaultBalanceChangeEntity(
  event: ethereum.Event,
  vaultEntityId: Bytes,
  oldVaultBalance: BigInt,
  amount: BigInt,
  type: VaultBalanceChangeType
): TradeVaultBalanceChange {
  let tradeVaultBalanceChange = new TradeVaultBalanceChange(eventId(event));
  tradeVaultBalanceChange.amount = amount;
  tradeVaultBalanceChange.oldVaultBalance = oldVaultBalance;
  if (type == VaultBalanceChangeType.CREDIT) {
    tradeVaultBalanceChange.newVaultBalance = oldVaultBalance.plus(amount);
  } else {
    tradeVaultBalanceChange.newVaultBalance = oldVaultBalance.minus(amount);
  }
  tradeVaultBalanceChange.vault = vaultEntityId;
  tradeVaultBalanceChange.trade;

  tradeVaultBalanceChange.save();
  return tradeVaultBalanceChange;
}
