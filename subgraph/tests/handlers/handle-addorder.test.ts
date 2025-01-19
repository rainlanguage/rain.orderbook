import {
  test,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  assert,
} from "matchstick-as";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { Evaluable, IO, createAddOrderEvent } from "../event-mocks.test";
import { makeOrderId } from "../../src/order";
import { vaultEntityId } from "../../src/vault";
import { createMockERC20Functions } from "../erc20.test";
import { handleAddOrder } from "../../src/handlers";

describe("Add and remove orders", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleAddOrder()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );
    createMockERC20Functions(
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    let owner = Address.fromString(
      "0x1234567890123456789012345678901234567890"
    );
    let input = new IO(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(18),
      BigInt.fromI32(1)
    );
    let output = new IO(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(18),
      BigInt.fromI32(1)
    );

    let event = createAddOrderEvent(
      owner,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      [input],
      [output],
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      new Evaluable(
        Address.fromString("0x1234567890123456789012345678901234567890"),
        Address.fromString("0x0987654321098765432109876543210987654321"),
        Bytes.fromHexString("0x1234567890123456789012345678901234567890")
      )
    );

    handleAddOrder(event);

    // we should have an orderbook entity
    assert.entityCount("Orderbook", 1);
    assert.fieldEquals(
      "Orderbook",
      event.address.toHexString(),
      "id",
      event.address.toHexString()
    );

    let id = makeOrderId(
      event.address,
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Order", 1);
    assert.fieldEquals("Order", id.toHexString(), "active", "true");
    assert.fieldEquals(
      "Order",
      id.toHexString(),
      "orderHash",
      "0x0987654321098765432109876543210987654321"
    );
    assert.fieldEquals(
      "Order",
      id.toHexString(),
      "owner",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Order",
      id.toHexString(),
      "nonce",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Order",
      id.toHexString(),
      "orderBytes",
      "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001c012345678901234567890123456789012345678900000000000000000000000000000000000000000000000001234567890123456789012345678901234567890000000000000000000000000098765432109876543210987654321098765432100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000014123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001234567890123456789012345678901234567890000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000098765432109876543210987654321098765432100000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001"
    );
    assert.fieldEquals(
      "Order",
      id.toHexString(),
      "timestampAdded",
      event.block.timestamp.toString()
    );

    // we should also have two new empty vaults
    assert.entityCount("Vault", 2);

    let inputVaultId = vaultEntityId(
      event.address,
      owner,
      input.vaultId,
      input.token
    );
    assert.fieldEquals("Vault", inputVaultId.toHexString(), "balance", "0");

    let outputVaultId = vaultEntityId(
      event.address,
      owner,
      output.vaultId,
      output.token
    );
    assert.fieldEquals("Vault", outputVaultId.toHexString(), "balance", "0");
  });
});
