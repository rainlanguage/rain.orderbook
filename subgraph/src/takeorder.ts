import { Bytes, ethereum } from "@graphprotocol/graph-ts";
import { TakeOrderV2 } from "../generated/OrderBook/OrderBook";
import { TakeOrder } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTransactionEntity } from "./transaction";
import {
  VaultBalanceChangeType,
  handleVaultBalanceChange,
  vaultEntityId,
} from "./vault";
import { createTradeVaultBalanceChangeEntity } from "./tradevaultbalancechange";
import { createTradeEntity } from "./trade";

export function handleTakeOrder(event: TakeOrderV2): void {
  let order = event.params.config.order;

  // Debit the output vault
  let orderOutputIO =
    order.validOutputs[event.params.config.outputIOIndex.toU32()];

  let oldOutputVaultBalance = handleVaultBalanceChange(
    orderOutputIO.vaultId,
    orderOutputIO.token,
    event.params.input, // input for the taker is the output amount for the vault
    order.owner,
    VaultBalanceChangeType.DEBIT
  );

  let outputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    vaultEntityId(order.owner, orderOutputIO.vaultId, orderOutputIO.token),
    oldOutputVaultBalance,
    event.params.input,
    VaultBalanceChangeType.DEBIT
  );

  let orderInputIO =
    order.validInputs[event.params.config.inputIOIndex.toU32()];

  // Credit the input vault
  let oldInputVaultBalance = handleVaultBalanceChange(
    orderInputIO.vaultId,
    orderInputIO.token,
    event.params.output, // output for the taker is the input amount for the vault
    order.owner,
    VaultBalanceChangeType.CREDIT
  );

  let inputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    vaultEntityId(order.owner, orderInputIO.vaultId, orderInputIO.token),
    oldInputVaultBalance,
    event.params.output,
    VaultBalanceChangeType.CREDIT
  );

  createTradeEntity(
    event,
    Bytes.empty(),
    inputVaultBalanceChange,
    outputVaultBalanceChange
  );

  createTakeOrderEntity(event);
}

export function createTakeOrderEntity(event: TakeOrderV2): void {
  let takeOrder = new TakeOrder(eventId(event));
  takeOrder.inputAmount = event.params.input;
  takeOrder.outputAmount = event.params.output;
  takeOrder.sender = event.params.sender;
  takeOrder.transaction = createTransactionEntity(event);
  takeOrder.takeOrderConfigBytes = ethereum.encode(event.parameters[1].value)!;
  takeOrder.save();
}
