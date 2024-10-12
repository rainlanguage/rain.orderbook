import { AfterClear, ClearV2 } from "../generated/OrderBook/OrderBook"
import { ClearTemporaryData } from "../generated/schema"
import { createOrderbookEntity } from "./orderbook"
import { createTradeEntity } from "./trade"
import { createTradeVaultBalanceChangeEntity } from "./tradevaultbalancechange"
import { handleVaultBalanceChange, vaultEntityId } from "./vault"
import { log } from "@graphprotocol/graph-ts"
import { BigInt, Bytes, crypto, ethereum, store } from "@graphprotocol/graph-ts"

export function orderHashFromClearEvent(order: ethereum.Value): Bytes {
  let orderHash = crypto.keccak256(ethereum.encode(order)!)
  return Bytes.fromByteArray(orderHash)
}

export function clearTemporaryDataEntityId(event: ethereum.Event): Bytes {
  return Bytes.fromByteArray(
    crypto.keccak256(event.transaction.hash.concat(event.address))
  )
}

export function handleClear(event: ClearV2): void {
  createOrderbookEntity(event)

  let clearTemporaryData = new ClearTemporaryData(
    clearTemporaryDataEntityId(event)
  )

  let aliceOrderHash = orderHashFromClearEvent(
    event.parameters[1].value.toTuple()[0]
  )
  let bobOrderHash = orderHashFromClearEvent(
    event.parameters[2].value.toTuple()[0]
  )

  let aliceInput =
    event.params.alice.validInputs[
      event.params.clearConfig.aliceInputIOIndex.toU32()
    ]
  let aliceOutput =
    event.params.alice.validOutputs[
      event.params.clearConfig.aliceOutputIOIndex.toU32()
    ]
  let bobInput =
    event.params.bob.validInputs[
      event.params.clearConfig.bobInputIOIndex.toU32()
    ]
  let bobOutput =
    event.params.bob.validOutputs[
      event.params.clearConfig.bobOutputIOIndex.toU32()
    ]

  clearTemporaryData.aliceOrderHash = aliceOrderHash
  clearTemporaryData.bobOrderHash = bobOrderHash

  clearTemporaryData.aliceAddress = event.params.alice.owner
  clearTemporaryData.bobAddress = event.params.bob.owner

  clearTemporaryData.aliceInputVaultId = aliceInput.vaultId
  clearTemporaryData.aliceOutputVaultId = aliceOutput.vaultId
  clearTemporaryData.bobInputVaultId = bobInput.vaultId
  clearTemporaryData.bobOutputVaultId = bobOutput.vaultId

  clearTemporaryData.aliceInputToken = aliceInput.token
  clearTemporaryData.aliceOutputToken = aliceOutput.token
  clearTemporaryData.bobInputToken = bobInput.token
  clearTemporaryData.bobOutputToken = bobOutput.token

  clearTemporaryData.aliceBounty = event.params.clearConfig.aliceBountyVaultId
  clearTemporaryData.bobBounty = event.params.clearConfig.bobBountyVaultId

  clearTemporaryData.save()
}

function createTrade(
  event: ethereum.Event,
  orderHash: Bytes,
  owner: Bytes,
  inputVaultId: BigInt,
  outputVaultId: BigInt,
  inputToken: Bytes,
  outputToken: Bytes,
  inputAmount: BigInt,
  outputAmount: BigInt
): void {
  let oldInputVaultBalance = handleVaultBalanceChange(
    event.address,
    inputVaultId,
    inputToken,
    inputAmount,
    owner
  )
  let oldOutputVaultBalance = handleVaultBalanceChange(
    event.address,
    outputVaultId,
    outputToken,
    outputAmount.neg(),
    owner
  )

  let inputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, inputVaultId, inputToken),
    oldInputVaultBalance,
    inputAmount
  )
  let outputVaultBalanceChange = createTradeVaultBalanceChangeEntity(
    event,
    orderHash,
    vaultEntityId(event.address, owner, outputVaultId, outputToken),
    oldOutputVaultBalance,
    outputAmount.neg()
  )

  createTradeEntity(
    event,
    orderHash,
    inputVaultBalanceChange,
    outputVaultBalanceChange
  )
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
  createOrderbookEntity(event)

  let clearTemporaryData = ClearTemporaryData.load(
    clearTemporaryDataEntityId(event)
  )
  if (clearTemporaryData) {
    createTrade(
      event,
      clearTemporaryData.aliceOrderHash,
      clearTemporaryData.aliceAddress,
      clearTemporaryData.aliceInputVaultId,
      clearTemporaryData.aliceOutputVaultId,
      clearTemporaryData.aliceInputToken,
      clearTemporaryData.aliceOutputToken,
      event.params.clearStateChange.aliceInput,
      event.params.clearStateChange.aliceOutput
    )
    createTrade(
      event,
      clearTemporaryData.bobOrderHash,
      clearTemporaryData.bobAddress,
      clearTemporaryData.bobInputVaultId,
      clearTemporaryData.bobOutputVaultId,
      clearTemporaryData.bobInputToken,
      clearTemporaryData.bobOutputToken,
      event.params.clearStateChange.bobInput,
      event.params.clearStateChange.bobOutput
    )

    // handle bounty vault changes
    const aliceBountyAmount = event.params.clearStateChange.aliceOutput.minus(
      event.params.clearStateChange.bobInput
    )
    const bobBountyAmount = event.params.clearStateChange.bobOutput.minus(
      event.params.clearStateChange.aliceInput
    )
    if (aliceBountyAmount.gt(BigInt.fromU32(0))) {
      handleVaultBalanceChange(
        event.address,
        clearTemporaryData.aliceBounty,
        clearTemporaryData.aliceOutputToken,
        aliceBountyAmount,
        event.params.sender
      )
    }
    if (bobBountyAmount.gt(BigInt.fromU32(0))) {
      handleVaultBalanceChange(
        event.address,
        clearTemporaryData.bobBounty,
        clearTemporaryData.bobOutputToken,
        bobBountyAmount,
        event.params.sender
      )
    }

    store.remove("ClearTemporaryData", clearTemporaryData.id.toHexString())
  } else {
    log.error("ClearTemporaryData not found for event {}", [
      event.transaction.hash.toHexString(),
    ])
  }
}
