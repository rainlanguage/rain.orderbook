import { AfterClear, ClearV2 } from "../generated/OrderBook/OrderBook";
import { ClearBounty, ClearTemporaryData } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTradeEntity } from "./trade";
import { createTradeVaultBalanceChangeEntity } from "./tradevaultbalancechange";
import { createTransactionEntity } from "./transaction";
import { handleVaultBalanceChange, vaultEntityId } from "./vault";
import { log } from "@graphprotocol/graph-ts";
import {
  BigInt,
  Bytes,
  crypto,
  ethereum,
  store,
} from "@graphprotocol/graph-ts";

export function orderHashFromClearEvent(
  event: ClearV2,
  isAlice: boolean
): Bytes {
  let orderHash = crypto.keccak256(
    ethereum.encode(event.parameters[isAlice ? 1 : 2].value.toTuple()[0])!
  );
  return Bytes.fromByteArray(orderHash);
}

export function clearTemporaryDataEntityId(event: ethereum.Event): Bytes {
  return Bytes.fromByteArray(
    crypto.keccak256(event.transaction.hash.concat(event.address))
  );
}

export function handleClear(event: ClearV2): void {
  createTransactionEntity(event);
  let clearTemporaryData = new ClearTemporaryData(
    clearTemporaryDataEntityId(event)
  );

  let aliceOrderHash = orderHashFromClearEvent(event, true);
  let bobOrderHash = orderHashFromClearEvent(event, false);

  let aliceInput =
    event.params.alice.validInputs[
      event.params.clearConfig.aliceInputIOIndex.toU32()
    ];
  let aliceOutput =
    event.params.alice.validOutputs[
      event.params.clearConfig.aliceOutputIOIndex.toU32()
    ];
  let bobInput =
    event.params.bob.validInputs[
      event.params.clearConfig.bobInputIOIndex.toU32()
    ];
  let bobOutput =
    event.params.bob.validOutputs[
      event.params.clearConfig.bobOutputIOIndex.toU32()
    ];

  clearTemporaryData.aliceOrderHash = aliceOrderHash;
  clearTemporaryData.bobOrderHash = bobOrderHash;

  clearTemporaryData.aliceAddress = event.params.alice.owner;
  clearTemporaryData.bobAddress = event.params.bob.owner;

  clearTemporaryData.aliceInputVaultId = aliceInput.vaultId;
  clearTemporaryData.aliceOutputVaultId = aliceOutput.vaultId;
  clearTemporaryData.bobInputVaultId = bobInput.vaultId;
  clearTemporaryData.bobOutputVaultId = bobOutput.vaultId;

  clearTemporaryData.aliceInputToken = aliceInput.token;
  clearTemporaryData.aliceOutputToken = aliceOutput.token;
  clearTemporaryData.bobInputToken = bobInput.token;
  clearTemporaryData.bobOutputToken = bobOutput.token;

  clearTemporaryData.aliceBounty = event.params.clearConfig.aliceBountyVaultId;
  clearTemporaryData.bobBounty = event.params.clearConfig.bobBountyVaultId;

  clearTemporaryData.save();
}

function createTrade(
  event: AfterClear,
  clearData: ClearTemporaryData,
  isAlice: boolean
): void {
  let owner = isAlice ? clearData.aliceAddress : clearData.bobAddress;

  let orderHash = isAlice ? clearData.aliceOrderHash : clearData.bobOrderHash;

  let input = isAlice
    ? event.params.clearStateChange.aliceInput
    : event.params.clearStateChange.bobInput;
  let output = isAlice
    ? event.params.clearStateChange.aliceOutput
    : event.params.clearStateChange.bobOutput;

  let inputVaultId = isAlice
    ? clearData.aliceInputVaultId
    : clearData.bobInputVaultId;
  let inputToken = isAlice
    ? clearData.aliceInputToken
    : clearData.bobInputToken;

  let outputVaultId = isAlice
    ? clearData.aliceOutputVaultId
    : clearData.bobOutputVaultId;
  let outputToken = isAlice
    ? clearData.aliceOutputToken
    : clearData.bobOutputToken;

  let oldInputVaultBalance = handleVaultBalanceChange(
    event.address,
    inputVaultId,
    inputToken,
    input,
    owner
  );
  let oldOutputVaultBalance = handleVaultBalanceChange(
    event.address,
    outputVaultId,
    outputToken,
    output.neg(),
    owner
  );

  let inputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, inputVaultId, inputToken),
    oldInputVaultBalance,
    input
  );
  let outputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, outputVaultId, outputToken),
    oldOutputVaultBalance,
    output.neg()
  );

  createTradeEntity(
    event,
    orderHash,
    inputVaultBalanceChange,
    outputVaultBalanceChange
  );
}

export function clearBountyId(event: AfterClear, vaultEntityId: Bytes): Bytes {
  let bytes = eventId(event).concat(vaultEntityId);
  return Bytes.fromByteArray(crypto.keccak256(bytes));
}

export function createClearBountyEntity(
  event: AfterClear,
  vaultEntityId: Bytes,
  oldVaultBalance: BigInt,
  amount: BigInt
): ClearBounty {
  let clearBounty = new ClearBounty(clearBountyId(event, vaultEntityId));
  clearBounty.orderbook = event.address;
  clearBounty.amount = amount;
  clearBounty.oldVaultBalance = oldVaultBalance;
  clearBounty.newVaultBalance = oldVaultBalance.plus(amount);
  clearBounty.vault = vaultEntityId;
  clearBounty.timestamp = event.block.timestamp;
  clearBounty.transaction = event.transaction.hash;
  clearBounty.sender = event.params.sender;
  clearBounty.save();
  return clearBounty;
}

export function handleClearBounty(
  event: AfterClear,
  clearTemporaryData: ClearTemporaryData
): void {
  const aliceBountyAmount = event.params.clearStateChange.aliceOutput.minus(
    event.params.clearStateChange.bobInput
  );
  const bobBountyAmount = event.params.clearStateChange.bobOutput.minus(
    event.params.clearStateChange.aliceInput
  );
  if (aliceBountyAmount.gt(BigInt.fromU32(0))) {
    const oldBalance = handleVaultBalanceChange(
      event.address,
      clearTemporaryData.aliceBounty,
      clearTemporaryData.aliceOutputToken,
      aliceBountyAmount,
      event.params.sender
    );
    createClearBountyEntity(
      event,
      vaultEntityId(
        event.address,
        event.params.sender,
        clearTemporaryData.aliceBounty,
        clearTemporaryData.aliceOutputToken
      ),
      oldBalance,
      aliceBountyAmount
    );
  }
  if (bobBountyAmount.gt(BigInt.fromU32(0))) {
    const oldBalance = handleVaultBalanceChange(
      event.address,
      clearTemporaryData.bobBounty,
      clearTemporaryData.bobOutputToken,
      bobBountyAmount,
      event.params.sender
    );
    createClearBountyEntity(
      event,
      vaultEntityId(
        event.address,
        event.params.sender,
        clearTemporaryData.bobBounty,
        clearTemporaryData.bobOutputToken
      ),
      oldBalance,
      bobBountyAmount
    );
  }
}

export function handleAfterClear(event: AfterClear): void {
  let clearTemporaryData = ClearTemporaryData.load(
    clearTemporaryDataEntityId(event)
  );
  if (clearTemporaryData) {
    createTrade(event, clearTemporaryData, true);
    createTrade(event, clearTemporaryData, false);
    handleClearBounty(event, clearTemporaryData);
    store.remove("ClearTemporaryData", clearTemporaryData.id.toHexString());
  } else {
    log.error("ClearTemporaryData not found for event {}", [
      event.transaction.hash.toHexString(),
    ]);
  }
}
