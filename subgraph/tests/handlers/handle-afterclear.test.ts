import { makeTradeId } from "../../src/trade";
import { makeOrderId } from "../../src/order";
import { Clear } from "../../generated/schema";
import { vaultEntityId } from "../../src/vault";
import { eventId } from "../../src/interfaces/event";
import { createMockERC20Functions } from "../erc20.test";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { tradeVaultBalanceChangeId } from "../../src/tradevaultbalancechange";
import {
  handleAfterClear,
  handleClear,
  getOrdersHash,
  makeClearBountyId,
} from "../../src/clear";
import {
  IO,
  Evaluable,
  createOrder,
  createClearEvent,
  createAfterClearEvent,
} from "../event-mocks.test";
import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";

const alice = Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce");
const bob = Address.fromString("0x813aef302Ebad333EDdef619C6f8eD7FeF51BA7c");

const aliceVaultId = BigInt.fromString("1");
const bobVaultId = BigInt.fromString("2");
const aliceBountyVaultId = BigInt.fromString("8");
const bobBountyVaultId = BigInt.fromString("9");

const token1 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB");
const token2 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc");
const token3 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Ba");

describe("Handle AfterClear", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleAfterClear()", () => {
    createMockERC20Functions(token1);
    createMockERC20Functions(token2);
    createMockERC20Functions(token3);

    let aliceOutputAmount = BigInt.fromString("10");
    let bobOutputAmount = BigInt.fromString("20");
    let aliceInputAmount = BigInt.fromString("15");
    let bobInputAmount = BigInt.fromString("10");

    let evaluable = new Evaluable(
      Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
      Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890")
    );
    let nonce = Bytes.fromHexString(
      "0xbce73059f54ada335f7283df99f81d42a3f2d09527eade865627e26cd756b748"
    );

    let clearEvent = createClearEvent(
      alice,
      createOrder(
        alice,
        evaluable,
        [new IO(token1, BigInt.fromString("18"), aliceVaultId)],
        [new IO(token2, BigInt.fromString("18"), aliceVaultId)],
        nonce
      ),
      createOrder(
        bob,
        evaluable,
        [new IO(token2, BigInt.fromString("18"), bobVaultId)],
        [
          new IO(token3, BigInt.fromString("18"), bobVaultId),
          new IO(token1, BigInt.fromString("18"), bobVaultId),
        ],
        nonce
      ),
      BigInt.fromString("0"),
      BigInt.fromString("0"),
      BigInt.fromString("0"),
      BigInt.fromString("1"),
      aliceBountyVaultId,
      bobBountyVaultId
    );

    let afterClearEvent = createAfterClearEvent(
      alice,
      aliceOutputAmount,
      bobOutputAmount,
      aliceInputAmount,
      bobInputAmount
    );

    let id = eventId(afterClearEvent).toHexString();
    let orderHashes = getOrdersHash(clearEvent);

    handleClear(clearEvent);
    assert.entityCount("ClearTemporaryData", 1);
    handleAfterClear(afterClearEvent);
    assert.entityCount("ClearTemporaryData", 0); // should be deleted by now

    // Clear entity
    assert.entityCount("Clear", 1);
    assert.fieldEquals("Clear", id, "sender", alice.toHexString()); // sender
    // alice
    assert.fieldEquals(
      "Clear",
      id,
      "aliceInputAmount",
      aliceInputAmount.toString()
    );
    assert.fieldEquals(
      "Clear",
      id,
      "aliceOutputAmount",
      aliceOutputAmount.toString()
    );
    assert.fieldEquals(
      "Clear",
      id,
      "aliceBountyAmount",
      aliceOutputAmount.minus(bobInputAmount).toString()
    );
    // bob
    assert.fieldEquals(
      "Clear",
      id,
      "bobInputAmount",
      bobInputAmount.toString()
    );
    assert.fieldEquals(
      "Clear",
      id,
      "bobOutputAmount",
      bobOutputAmount.toString()
    );
    assert.fieldEquals(
      "Clear",
      id,
      "bobBountyAmount",
      bobOutputAmount.minus(aliceInputAmount).toString()
    );

    // bounty
    let bountyVaultId = vaultEntityId(
      afterClearEvent.address,
      afterClearEvent.params.sender,
      bobBountyVaultId,
      token1
    );
    let clearBountyId = makeClearBountyId(
      afterClearEvent,
      bountyVaultId
    ).toHexString();
    // Clear entity should only have bob bounty and not alice
    assert.assertTrue(
      !Clear.load(Bytes.fromHexString(id))!.aliceBountyVaultBalanceChange
    );
    assert.fieldEquals(
      "Clear",
      id,
      "bobBountyVaultBalanceChange",
      clearBountyId
    );

    // ClearBounty entity
    assert.entityCount("ClearBounty", 1); // should only have 1 bounty
    assert.fieldEquals(
      "ClearBounty",
      clearBountyId,
      "sender",
      alice.toHexString()
    );
    assert.fieldEquals(
      "ClearBounty",
      clearBountyId,
      "amount",
      bobOutputAmount.minus(aliceInputAmount).toString()
    );
    assert.fieldEquals(
      "ClearBounty",
      clearBountyId,
      "newVaultBalance",
      bobOutputAmount.minus(aliceInputAmount).toString()
    );
    assert.fieldEquals(
      "ClearBounty",
      clearBountyId,
      "oldVaultBalance",
      BigInt.fromString("0").toString()
    );
    assert.fieldEquals(
      "ClearBounty",
      clearBountyId,
      "vault",
      bountyVaultId.toHexString()
    );

    // trades and trade vault balance changes
    assert.entityCount("Trade", 2);
    assert.entityCount("TradeVaultBalanceChange", 4);

    // alice trade and balance change
    let aliceInputVaultEntityId = vaultEntityId(
      afterClearEvent.address,
      alice,
      aliceVaultId,
      token1
    );
    let aliceOutputVaultEntityId = vaultEntityId(
      afterClearEvent.address,
      alice,
      aliceVaultId,
      token2
    );
    let aliceInputVaultBalanceChangeId = tradeVaultBalanceChangeId(
      afterClearEvent,
      aliceInputVaultEntityId
    ).toHexString();
    let aliceOutputVaultBalanceChangeId = tradeVaultBalanceChangeId(
      afterClearEvent,
      aliceOutputVaultEntityId
    ).toHexString();
    let aliceTradeId = makeTradeId(
      afterClearEvent,
      orderHashes[0]
    ).toHexString();
    assert.fieldEquals(
      "Trade",
      aliceTradeId,
      "order",
      makeOrderId(afterClearEvent.address, orderHashes[0]).toHexString()
    );
    assert.fieldEquals(
      "Trade",
      aliceTradeId,
      "inputVaultBalanceChange",
      aliceInputVaultBalanceChangeId
    );
    assert.fieldEquals(
      "Trade",
      aliceTradeId,
      "outputVaultBalanceChange",
      aliceOutputVaultBalanceChangeId
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      aliceInputVaultBalanceChangeId,
      "amount",
      aliceInputAmount.toString()
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      aliceOutputVaultBalanceChangeId,
      "amount",
      aliceOutputAmount.neg().toString()
    );

    // bob trade and balance change
    let bobInputVaultEntityId = vaultEntityId(
      afterClearEvent.address,
      bob,
      bobVaultId,
      token2
    );
    let bobOutputVaultEntityId = vaultEntityId(
      afterClearEvent.address,
      bob,
      bobVaultId,
      token1
    );
    let bobInputVaultBalanceChangeId = tradeVaultBalanceChangeId(
      afterClearEvent,
      bobInputVaultEntityId
    ).toHexString();
    let bobOutputVaultBalanceChangeId = tradeVaultBalanceChangeId(
      afterClearEvent,
      bobOutputVaultEntityId
    ).toHexString();
    let bobTradeId = makeTradeId(afterClearEvent, orderHashes[1]).toHexString();
    assert.fieldEquals(
      "Trade",
      bobTradeId,
      "order",
      makeOrderId(afterClearEvent.address, orderHashes[1]).toHexString()
    );
    assert.fieldEquals(
      "Trade",
      bobTradeId,
      "inputVaultBalanceChange",
      bobInputVaultBalanceChangeId
    );
    assert.fieldEquals(
      "Trade",
      bobTradeId,
      "outputVaultBalanceChange",
      bobOutputVaultBalanceChangeId
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      bobInputVaultBalanceChangeId,
      "amount",
      bobInputAmount.toString()
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      bobOutputVaultBalanceChangeId,
      "amount",
      bobOutputAmount.neg().toString()
    );
  });
});
