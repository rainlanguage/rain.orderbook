import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as"
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts"
import {
  clearTemporaryDataEntityId,
  handleAfterClear,
  handleClear,
} from "../../src/clear"
import {
  AfterClearClearStateChangeStruct,
  ClearV2ClearConfigStruct,
  ClearV2Struct,
  createAfterClearEvent,
  createClearEvent,
  createDepositEvent,
  Evaluable,
  IO,
} from "../event-mocks.test"
import { createMockERC20Functions } from "../erc20.test"
import { ClearTemporaryData, Vault } from "../../generated/schema"
import { vaultEntityId } from "../../src/vault"
import { handleDeposit } from "../../src/deposit"

const alice = Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce")
const bob = Address.fromString("0x813aef302Ebad333EDdef619C6f8eD7FeF51BA7c")

const token1 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB")
const token2 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc")
const token3 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Ba")

describe("Clear", () => {
  afterEach(() => {
    clearStore()
    clearInBlockStore()
  })

  test("clearEvent and afterClearEvent", () => {
    createMockERC20Functions(token1)
    createMockERC20Functions(token2)
    createMockERC20Functions(token3)

    // Alice input token
    handleDeposit(
      createDepositEvent(
        alice,
        token1,
        BigInt.fromI32(1),
        BigInt.fromString("10000")
      )
    )
    // Alice output token
    handleDeposit(
      createDepositEvent(
        alice,
        token2,
        BigInt.fromI32(1),
        BigInt.fromString("10000")
      )
    )
    // Bob input token
    handleDeposit(
      createDepositEvent(
        bob,
        token2,
        BigInt.fromI32(2),
        BigInt.fromString("10000")
      )
    )
    // Bob output tokens
    handleDeposit(
      createDepositEvent(
        bob,
        token3,
        BigInt.fromI32(2),
        BigInt.fromString("10000")
      )
    )
    handleDeposit(
      createDepositEvent(
        bob,
        token1,
        BigInt.fromI32(2),
        BigInt.fromString("10000")
      )
    )

    let clearEvent = createClearEvent(
      alice,
      new ClearV2Struct(
        alice,
        new Evaluable(
          Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
          Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
          Bytes.fromHexString("0x1234567890123456789012345678901234567890")
        ),
        [new IO(token1, BigInt.fromI32(18), BigInt.fromI32(1))],
        [new IO(token2, BigInt.fromI32(18), BigInt.fromI32(1))],
        Bytes.fromHexString(
          "0xbce73059f54ada335f7283df99f81d42a3f2d09527eade865627e26cd756b748"
        )
      ),
      new ClearV2Struct(
        bob,
        new Evaluable(
          Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
          Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
          Bytes.fromHexString("0x1234567890123456789012345678901234567890")
        ),
        [new IO(token2, BigInt.fromI32(18), BigInt.fromI32(2))],
        [
          new IO(token3, BigInt.fromI32(18), BigInt.fromI32(2)),
          new IO(token1, BigInt.fromI32(18), BigInt.fromI32(2)),
        ],
        Bytes.fromHexString(
          "0x9c8176f8e6e02b5f02eee226ff7066d2474bdc50f89bd15dca539240e0cb1788"
        )
      ),
      new ClearV2ClearConfigStruct(
        BigInt.fromI32(0),
        BigInt.fromI32(0),
        BigInt.fromI32(0),
        BigInt.fromI32(1),
        BigInt.fromI32(1),
        BigInt.fromI32(1)
      )
    )

    let aliceInput = clearEvent.params.alice.validInputs[0]
    assert.addressEquals(aliceInput.token, token1)
    let aliceOutput = clearEvent.params.alice.validOutputs[0]
    assert.addressEquals(aliceOutput.token, token2)

    let bobInput = clearEvent.params.bob.validInputs[0]
    assert.addressEquals(bobInput.token, token2)
    let bobOutput = clearEvent.params.bob.validOutputs[1]
    assert.addressEquals(bobOutput.token, token1)

    let id = clearTemporaryDataEntityId(clearEvent)
    handleClear(clearEvent)

    assert.entityCount("ClearTemporaryData", 1)
    let clearTemporaryData = ClearTemporaryData.load(id)
    assert.assertNotNull(clearTemporaryData)
    if (clearTemporaryData == null) {
      return
    }
    assert.bytesEquals(clearTemporaryData.aliceAddress, alice)
    assert.bytesEquals(clearTemporaryData.bobAddress, bob)
    assert.bigIntEquals(clearTemporaryData.aliceInputVaultId, BigInt.fromI32(1))
    assert.bigIntEquals(
      clearTemporaryData.aliceOutputVaultId,
      BigInt.fromI32(1)
    )
    assert.bigIntEquals(clearTemporaryData.bobInputVaultId, BigInt.fromI32(2))
    assert.bigIntEquals(clearTemporaryData.bobOutputVaultId, BigInt.fromI32(2))
    assert.bytesEquals(clearTemporaryData.aliceInputToken, token1)
    assert.bytesEquals(clearTemporaryData.aliceOutputToken, token2)
    assert.bytesEquals(clearTemporaryData.bobInputToken, token2)
    assert.bytesEquals(clearTemporaryData.bobOutputToken, token1)

    let afterClearEvent = createAfterClearEvent(
      alice,
      new AfterClearClearStateChangeStruct(
        BigInt.fromString("1000"),
        BigInt.fromString("1247"),
        BigInt.fromString("1130"),
        BigInt.fromString("1000")
      )
    )

    id = clearTemporaryDataEntityId(afterClearEvent)
    handleAfterClear(afterClearEvent)

    assert.entityCount("ClearTemporaryData", 0)
    assert.entityCount("Trade", 2)
    assert.entityCount("TradeVaultBalanceChange", 4)

    let aliceInputVault = Vault.load(
      vaultEntityId(
        afterClearEvent.address,
        clearEvent.params.alice.owner,
        aliceInput.vaultId,
        aliceInput.token
      )
    )
    assert.assertNotNull(aliceInputVault)
    if (aliceInputVault == null) {
      return
    }
    assert.bigIntEquals(aliceInputVault.balance, BigInt.fromString("11130"))

    let aliceOutputVault = Vault.load(
      vaultEntityId(
        clearEvent.address,
        clearEvent.params.alice.owner,
        aliceOutput.vaultId,
        aliceOutput.token
      )
    )
    assert.assertNotNull(aliceOutputVault)
    if (aliceOutputVault == null) {
      return
    }
    assert.bigIntEquals(aliceOutputVault.balance, BigInt.fromString("9000"))

    let bobVault = Vault.load(
      vaultEntityId(
        clearEvent.address,
        clearEvent.params.bob.owner,
        bobOutput.vaultId,
        bobOutput.token
      )
    )
    assert.assertNotNull(bobVault)
    if (bobVault == null) {
      return
    }
    assert.bigIntEquals(
      bobVault.balance,
      BigInt.fromString("1168184852026402880")
    )
  })
})
