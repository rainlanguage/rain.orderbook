import { AfterClear, ClearV2 } from "../generated/OrderBook/OrderBook";
import { Clear, ClearBounty, ClearTemporaryData } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { createTradeEntity } from "./trade";
import { createTradeVaultBalanceChangeEntity } from "./tradevaultbalancechange";
import { handleVaultBalanceChange, vaultEntityId } from "./vault";
import { log } from "@graphprotocol/graph-ts";
import {
  BigInt,
  Bytes,
  crypto,
  ethereum,
  store,
} from "@graphprotocol/graph-ts";

export function makeClearEventId(event: ethereum.Event): Bytes {
  return Bytes.fromByteArray(
    crypto.keccak256(event.address.concat(event.transaction.hash))
  );
}

export function getOrdersHash(event: ClearV2): Bytes[] {
  return [
    Bytes.fromByteArray(
      crypto.keccak256(ethereum.encode(event.parameters[1].value)!)
    ),
    Bytes.fromByteArray(
      crypto.keccak256(ethereum.encode(event.parameters[2].value)!)
    ),
  ];
}

export function makeClearBountyId(
  event: AfterClear,
  vaultEntityId: Bytes
): Bytes {
  return Bytes.fromByteArray(
    crypto.keccak256(vaultEntityId.concat(eventId(event)))
  );
}

export function createTrade(
  event: AfterClear,
  owner: Bytes,
  orderHash: Bytes,
  inputToken: Bytes,
  inputVaultId: BigInt,
  inputAmount: BigInt,
  outputToken: Bytes,
  outputVaultId: BigInt,
  outputAmount: BigInt
): void {
  let oldInputVaultBalance = handleVaultBalanceChange(
    event.address,
    inputVaultId,
    inputToken,
    inputAmount,
    owner
  );
  let inputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, inputVaultId, inputToken),
    oldInputVaultBalance,
    inputAmount
  );

  let oldOutputVaultBalance = handleVaultBalanceChange(
    event.address,
    outputVaultId,
    outputToken,
    outputAmount.neg(),
    owner
  );
  let outputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, outputVaultId, outputToken),
    oldOutputVaultBalance,
    outputAmount.neg()
  );

  createTradeEntity(
    event,
    orderHash,
    inputVaultBalanceChange,
    outputVaultBalanceChange
  );
}

export function createClearEntity(
  event: AfterClear,
  aliceBountyAmount: BigInt,
  bobBountyAmount: BigInt,
  aliceClearBounty: Bytes | null,
  bobClearBounty: Bytes | null
): void {
  const zero = BigInt.fromU32(0);
  let clear = new Clear(eventId(event));
  clear.orderbook = event.address;
  clear.aliceInputAmount = event.params.clearStateChange.aliceInput;
  clear.aliceOutputAmount = event.params.clearStateChange.aliceOutput;
  clear.aliceBountyAmount = aliceBountyAmount.gt(zero)
    ? aliceBountyAmount
    : zero;

  clear.bobInputAmount = event.params.clearStateChange.bobInput;
  clear.bobOutputAmount = event.params.clearStateChange.bobOutput;
  clear.bobBountyAmount = bobBountyAmount.gt(zero) ? bobBountyAmount : zero;

  if (aliceClearBounty) {
    clear.aliceBountyVaultBalanceChange = aliceClearBounty;
  }
  if (bobClearBounty) {
    clear.bobBountyVaultBalanceChange = bobClearBounty;
  }

  clear.sender = event.params.sender;
  clear.transaction = event.transaction.hash;
  clear.save();
}

export function createClearBountyEntity(
  event: AfterClear,
  vaultEntityId: Bytes,
  oldVaultBalance: BigInt,
  amount: BigInt
): ClearBounty {
  let clearBounty = new ClearBounty(makeClearBountyId(event, vaultEntityId));
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
  let aliceClearBounty: Bytes | null = null;
  let bobClearBounty: Bytes | null = null;
  let aliceBountyAmount = event.params.clearStateChange.aliceOutput.minus(
    event.params.clearStateChange.bobInput
  );
  let bobBountyAmount = event.params.clearStateChange.bobOutput.minus(
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
    aliceClearBounty = createClearBountyEntity(
      event,
      vaultEntityId(
        event.address,
        event.params.sender,
        clearTemporaryData.aliceBounty,
        clearTemporaryData.aliceOutputToken
      ),
      oldBalance,
      aliceBountyAmount
    ).id;
  }
  if (bobBountyAmount.gt(BigInt.fromU32(0))) {
    const oldBalance = handleVaultBalanceChange(
      event.address,
      clearTemporaryData.bobBounty,
      clearTemporaryData.bobOutputToken,
      bobBountyAmount,
      event.params.sender
    );
    bobClearBounty = createClearBountyEntity(
      event,
      vaultEntityId(
        event.address,
        event.params.sender,
        clearTemporaryData.bobBounty,
        clearTemporaryData.bobOutputToken
      ),
      oldBalance,
      bobBountyAmount
    ).id;
  }

  createClearEntity(
    event,
    aliceBountyAmount,
    bobBountyAmount,
    aliceClearBounty,
    bobClearBounty
  );
}

export function handleClear(event: ClearV2): void {
  let clearTemporaryData = new ClearTemporaryData(makeClearEventId(event));

  let hashes = getOrdersHash(event);
  let aliceOrderHash = hashes[0];
  let bobOrderHash = hashes[1];

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

export function handleAfterClear(event: AfterClear): void {
  let clearTemporaryData = ClearTemporaryData.load(makeClearEventId(event));
  if (clearTemporaryData) {
    // alice
    createTrade(
      event,
      clearTemporaryData.aliceAddress,
      clearTemporaryData.aliceOrderHash,
      clearTemporaryData.aliceInputToken,
      clearTemporaryData.aliceInputVaultId,
      event.params.clearStateChange.aliceInput,
      clearTemporaryData.aliceOutputToken,
      clearTemporaryData.aliceOutputVaultId,
      event.params.clearStateChange.aliceOutput
    );
    // bob
    createTrade(
      event,
      clearTemporaryData.bobAddress,
      clearTemporaryData.bobOrderHash,
      clearTemporaryData.bobInputToken,
      clearTemporaryData.bobInputVaultId,
      event.params.clearStateChange.bobInput,
      clearTemporaryData.bobOutputToken,
      clearTemporaryData.bobOutputVaultId,
      event.params.clearStateChange.bobOutput
    );

    // bounty and clear entity
    handleClearBounty(event, clearTemporaryData);

    store.remove("ClearTemporaryData", clearTemporaryData.id.toHexString());
  } else {
    log.error("ClearTemporaryData not found for event {}", [
      event.transaction.hash.toHexString(),
    ]);
  }
}
