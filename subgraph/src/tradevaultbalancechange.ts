import { BigInt, Bytes, ethereum } from "@graphprotocol/graph-ts";
import { TradeVaultBalanceChange } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { VaultBalanceChangeType } from "./vault";
import { makeTradeId } from "./trade";
import { orderHashFromTakeOrderEvent } from "./takeorder";

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
  amount: BigInt,
  type: VaultBalanceChangeType
): TradeVaultBalanceChange {
  let tradeVaultBalanceChange = new TradeVaultBalanceChange(
    tradeVaultBalanceChangeId(event, vaultEntityId)
  );
  tradeVaultBalanceChange.amount = amount;
  tradeVaultBalanceChange.oldVaultBalance = oldVaultBalance;
  if (type == VaultBalanceChangeType.CREDIT) {
    tradeVaultBalanceChange.newVaultBalance = oldVaultBalance.plus(amount);
  } else {
    tradeVaultBalanceChange.newVaultBalance = oldVaultBalance.minus(amount);
  }
  tradeVaultBalanceChange.vault = vaultEntityId;
  tradeVaultBalanceChange.trade = makeTradeId(event, orderHash);
  tradeVaultBalanceChange.save();
  return tradeVaultBalanceChange;
}
