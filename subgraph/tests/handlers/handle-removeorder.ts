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
  createRemoveOrderEvent,
} from "../event-mocks.test";
import { handleAddOrder, handleRemoveOrder } from "../../src/order";

describe("Add and remove orders", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleRemoveOrder()", () => {
    // First we need to add an order
    let event = createAddOrderEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
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

    // Now we can remove the order
    let removeEvent = createRemoveOrderEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
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

    handleRemoveOrder(removeEvent);

    assert.entityCount("Order", 1);
    assert.fieldEquals(
      "Order",
      "0x0987654321098765432109876543210987654321",
      "active",
      "false"
    );

    // if we add the order again, it should be active
    handleAddOrder(event);

    assert.entityCount("Order", 1);

    assert.fieldEquals(
      "Order",
      "0x0987654321098765432109876543210987654321",
      "active",
      "true"
    );

    // if we remove the order again, it should be inactive

    handleRemoveOrder(removeEvent);

    assert.entityCount("Order", 1);

    assert.fieldEquals(
      "Order",
      "0x0987654321098765432109876543210987654321",
      "active",
      "false"
    );
  });
});
