import { Bytes, ethereum } from "@graphprotocol/graph-ts";
import { Trade, TradeVaultBalanceChange } from "../generated/schema";
import { eventId } from "./interfaces/event";

export function makeTradeId(event: ethereum.Event, orderHash: Bytes): Bytes {
  return eventId(event).concat(orderHash);
}

export function createTradeEntity(
  event: ethereum.Event,
  orderHash: Bytes,
  inputVaultBalanceChange: TradeVaultBalanceChange,
  outputVaultBalanceChange: TradeVaultBalanceChange
): void {
  let trade = new Trade(makeTradeId(event, orderHash));
  trade.order = orderHash;
  trade.inputVaultBalanceChange = inputVaultBalanceChange.id;
  trade.outputVaultBalanceChange = outputVaultBalanceChange.id;
  trade.tradeEvent = eventId(event);
  trade.save();
}
