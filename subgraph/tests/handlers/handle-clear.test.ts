import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { makeClearEventId, handleClear, getOrdersHash } from "../../src/clear";
import {
  IO,
  Evaluable,
  createOrder,
  createClearEvent,
} from "../event-mocks.test";
import {
  test,
  assert,
  describe,
  afterEach,
  clearStore,
  clearInBlockStore,
} from "matchstick-as";

const alice = Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce");
const bob = Address.fromString("0x813aef302Ebad333EDdef619C6f8eD7FeF51BA7c");

const aliceVaultId = BigInt.fromI32(1);
const bobVaultId = BigInt.fromI32(2);
const aliceBountyVaultId = BigInt.fromI32(8);
const bobBountyVaultId = BigInt.fromI32(9);

const token1 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB");
const token2 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc");
const token3 = Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Ba");

describe("Handle Clear", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleClear()", () => {
    let evaluable = new Evaluable(
      Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
      Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890")
    );
    let nonce = Bytes.fromHexString(
      "0xbce73059f54ada335f7283df99f81d42a3f2d09527eade865627e26cd756b748"
    );

    let event = createClearEvent(
      alice,
      createOrder(
        alice,
        evaluable,
        [new IO(token1, BigInt.fromI32(18), aliceVaultId)],
        [new IO(token2, BigInt.fromI32(18), aliceVaultId)],
        nonce
      ),
      createOrder(
        bob,
        evaluable,
        [new IO(token2, BigInt.fromI32(18), bobVaultId)],
        [
          new IO(token3, BigInt.fromI32(18), bobVaultId),
          new IO(token1, BigInt.fromI32(18), bobVaultId),
        ],
        nonce
      ),
      BigInt.fromI32(0),
      BigInt.fromI32(0),
      BigInt.fromI32(0),
      BigInt.fromI32(1),
      aliceBountyVaultId,
      bobBountyVaultId
    );

    let id = makeClearEventId(event).toHexString();
    let orderHashes = getOrdersHash(event);
    handleClear(event);

    assert.entityCount("ClearTemporaryData", 1);

    // alice
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceAddress",
      alice.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceOrderHash",
      orderHashes[0].toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceInputToken",
      token1.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceOutputToken",
      token2.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceInputVaultId",
      aliceVaultId.toString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceOutputVaultId",
      aliceVaultId.toString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "aliceBounty",
      aliceBountyVaultId.toString()
    );

    // bob
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobAddress",
      bob.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobAddress",
      bob.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobOrderHash",
      orderHashes[1].toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobInputToken",
      token2.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobOutputToken",
      token1.toHexString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobInputVaultId",
      bobVaultId.toString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobOutputVaultId",
      bobVaultId.toString()
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id,
      "bobBounty",
      bobBountyVaultId.toString()
    );
  });
});
