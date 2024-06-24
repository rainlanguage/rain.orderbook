import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address, Bytes, ethereum } from "@graphprotocol/graph-ts";
import { createDepositEntity } from "../src/deposit";
import {
  Evaluable,
  IO,
  createAddOrderEvent,
  createDepositEvent,
  createTakeOrderEvent,
} from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";
import { createTakeOrderEntity } from "../src/takeorder";
import { eventId } from "../src/interfaces/event";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createTakeOrderEvent()", () => {
    let event = createTakeOrderEvent(
      Address.fromString("0x1111111111111111111111111111111111111111"),
      Address.fromString("0x2222222222222222222222222222222222222222"),
      [
        new IO(
          Address.fromString("0x3333333333333333333333333333333333333333"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      [
        new IO(
          Address.fromString("0x4444444444444444444444444444444444444444"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      Bytes.fromHexString("0x5555555555555555555555555555555555555555"),
      new Evaluable(
        Address.fromString("0x6666666666666666666666666666666666666666"),
        Address.fromString("0x7777777777777777777777777777777777777777"),
        Bytes.fromHexString("0x8888888888888888888888888888888888888888")
      ),
      BigInt.fromI32(1),
      BigInt.fromI32(1)
    );

    assert.addressEquals(
      event.params.sender,
      Address.fromString("0x1111111111111111111111111111111111111111")
    );
    assert.addressEquals(
      event.params.config.order.owner,
      Address.fromString("0x2222222222222222222222222222222222222222")
    );
    assert.bytesEquals(
      event.params.config.order.nonce,
      Bytes.fromHexString("0x5555555555555555555555555555555555555555")
    );
    assert.addressEquals(
      event.params.config.order.evaluable.interpreter,
      Address.fromString("0x6666666666666666666666666666666666666666")
    );
    assert.addressEquals(
      event.params.config.order.evaluable.store,
      Address.fromString("0x7777777777777777777777777777777777777777")
    );
    assert.bytesEquals(
      event.params.config.order.evaluable.bytecode,
      Bytes.fromHexString("0x8888888888888888888888888888888888888888")
    );
    let input = event.params.config.order.validInputs[0];
    assert.addressEquals(
      input.token,
      Address.fromString("0x3333333333333333333333333333333333333333")
    );
    assert.bigIntEquals(input.vaultId, BigInt.fromI32(1));
    assert.bigIntEquals(BigInt.fromI32(input.decimals), BigInt.fromI32(18));
    let output = event.params.config.order.validOutputs[0];
    assert.addressEquals(
      output.token,
      Address.fromString("0x4444444444444444444444444444444444444444")
    );
    assert.bigIntEquals(output.vaultId, BigInt.fromI32(1));
    assert.bigIntEquals(BigInt.fromI32(output.decimals), BigInt.fromI32(18));
    assert.bigIntEquals(event.params.input, BigInt.fromI32(1));
    assert.bigIntEquals(event.params.output, BigInt.fromI32(1));
  });

  test("createTakeOrderEntity()", () => {
    let event = createTakeOrderEvent(
      Address.fromString("0x1111111111111111111111111111111111111111"),
      Address.fromString("0x2222222222222222222222222222222222222222"),
      [
        new IO(
          Address.fromString("0x3333333333333333333333333333333333333333"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      [
        new IO(
          Address.fromString("0x4444444444444444444444444444444444444444"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      Bytes.fromHexString("0x5555555555555555555555555555555555555555"),
      new Evaluable(
        Address.fromString("0x6666666666666666666666666666666666666666"),
        Address.fromString("0x7777777777777777777777777777777777777777"),
        Bytes.fromHexString("0x8888888888888888888888888888888888888888")
      ),
      BigInt.fromI32(2),
      BigInt.fromI32(3)
    );

    createTakeOrderEntity(event);

    assert.entityCount("TakeOrder", 1);

    let id = eventId(event).toHexString();

    assert.fieldEquals(
      "TakeOrder",
      id,
      "sender",
      "0x1111111111111111111111111111111111111111"
    );
    assert.fieldEquals("TakeOrder", id, "inputAmount", "2");
    assert.fieldEquals("TakeOrder", id, "outputAmount", "3");
    assert.fieldEquals(
      "TakeOrder",
      id,
      "takeOrderConfigBytes",
      ethereum.encode(event.parameters[1].value)!.toHexString()
    );
  });
});
