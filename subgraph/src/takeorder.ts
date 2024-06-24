import { ethereum } from "@graphprotocol/graph-ts";
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
import { crypto } from "@graphprotocol/graph-ts";

export function handleTakeOrder(event: TakeOrderV2): void {
  let order = event.params.config.order;

  // Debit the output vault
  let orderOutput =
    order.validOutputs[event.params.config.outputIOIndex.toU32()];

  let oldOutputVaultBalance = handleVaultBalanceChange(
    orderOutput.vaultId,
    orderOutput.token,
    event.params.input, // input for the taker is the output amount for the vault
    order.owner,
    VaultBalanceChangeType.DEBIT
  );

  let outputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    vaultEntityId(order.owner, orderOutput.vaultId, orderOutput.token),
    oldOutputVaultBalance,
    event.params.input,
    VaultBalanceChangeType.DEBIT
  );

  // Credit the input vault
  let orderInput = order.validInputs[event.params.config.inputIOIndex.toU32()];

  let oldInputVaultBalance = handleVaultBalanceChange(
    orderInput.vaultId,
    orderInput.token,
    event.params.output, // output for the taker is the input amount for the vault
    order.owner,
    VaultBalanceChangeType.CREDIT
  );

  let inputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    vaultEntityId(order.owner, orderInput.vaultId, orderInput.token),
    oldInputVaultBalance,
    event.params.output,
    VaultBalanceChangeType.CREDIT
  );

  // hashing the raw bytes of the OrderV3
  let orderHash = crypto.keccak256(
    ethereum.encode(event.parameters[1].value.toTuple()[0])!
  );

  createTradeEntity(
    event,
    orderHash,
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
