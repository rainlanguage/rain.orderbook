import {
  test,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  assert,
} from "matchstick-as";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import {
  Evaluable,
  IO,
  createAddOrderEvent,
  createMetaEvent,
} from "../event-mocks.test";
import { handleAddOrder } from "../../src/order";
import { handleMeta } from "../../src/handlers";
import { createMockERC20Functions } from "../erc20.test";

describe("Add and remove orders", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleMeta() should not error if there is no order", () => {
    let event = createMetaEvent(
      // sender
      Address.fromString(
        "0x1234567890abcdef1234567890abcdef12345678"
      ) as Address,
      // subject
      BigInt.fromI32(1),
      // meta
      Bytes.fromHexString("0x1234567890abcdef1234567890abcdef12345678")
    );

    handleMeta(event);
  });

  test("handleMeta() should update the meta field of an order", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let orderHash = Bytes.fromHexString(
      "0x0987654321098765432109876543210987654321"
    );
    // first we need to create an order
    let event = createAddOrderEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      orderHash,
      [
        new IO(
          Address.fromString("0x1234567890123456789012345678901234567890"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      [
        new IO(
          Address.fromString("0x1234567890123456789012345678901234567890"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      new Evaluable(
        Address.fromString("0x1234567890123456789012345678901234567890"),
        Address.fromString("0x0987654321098765432109876543210987654321"),
        Bytes.fromHexString("0x1234567890123456789012345678901234567890")
      )
    );

    handleAddOrder(event);

    let metaEvent = createMetaEvent(
      // sender
      Address.fromBytes(
        Address.fromHexString("0x1234567890abcdef1234567890abcdef12345678")
      ),
      // subject
      BigInt.fromByteArray(orderHash),
      // meta
      Bytes.fromHexString("0x1234567890abcdef1234567890abcdef12345678")
    );

    handleMeta(metaEvent);

    // meta field on order should be updated
    assert.fieldEquals(
      "Order",
      orderHash.toHexString(),
      "meta",
      "0x1234567890abcdef1234567890abcdef12345678"
    );
  });
});
