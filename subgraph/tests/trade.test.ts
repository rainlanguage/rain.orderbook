import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  newMockEvent,
} from "matchstick-as";
import {
  BigInt,
  Address,
  Bytes,
  ethereum,
  crypto,
} from "@graphprotocol/graph-ts";
import { Evaluable, IO, createTakeOrderEvent } from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";
import { eventId } from "../src/interfaces/event";
import { createTradeEntity, makeTradeId } from "../src/trade";
import { TradeVaultBalanceChange } from "../generated/schema";
import { tradeVaultBalanceChangeId } from "../src/tradevaultbalancechange";
import { createMockERC20Functions } from "./erc20.test";
import { makeOrderId } from "../src/order";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("makeTradeId()", () => {
    let event = newMockEvent();
    event.logIndex = BigInt.fromI32(2);
    event.transaction.hash = Bytes.fromHexString(
      "0x1111111111111111111111111111111111111111111111111111111111111111"
    );

    let orderHash = Bytes.fromHexString(
      "0x3333333333333333333333333333333333333333"
    );

    let tradeId = Bytes.fromByteArray(
      crypto.keccak256(eventId(event).concat(orderHash))
    );

    assert.bytesEquals(
      tradeId,
      Bytes.fromHexString(
        "0x6e82bd6b67d3ab900f932bb0bd3cc4fb2c96e4a9e8cf0d4476271bea34bf226b"
      )
    );
  });

  test("createTradeEntity()", () => {
    createMockERC20Functions(
      Address.fromString("0x3333333333333333333333333333333333333333")
    );
    createMockERC20Functions(
      Address.fromString("0x4444444444444444444444444444444444444444")
    );

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

    let orderHash = Bytes.fromByteArray(
      crypto.keccak256(ethereum.encode(event.parameters[1].value)!)
    );

    let inputVaultBalanceChange = new TradeVaultBalanceChange(
      tradeVaultBalanceChangeId(
        event,
        vaultEntityId(
          event.address,
          Address.fromString("0x1111111111111111111111111111111111111111"),
          BigInt.fromU32(1),
          Address.fromString("0x3333333333333333333333333333333333333333")
        )
      )
    );

    let outputVaultBalanceChange = new TradeVaultBalanceChange(
      tradeVaultBalanceChangeId(
        event,
        vaultEntityId(
          event.address,
          Address.fromString("0x1111111111111111111111111111111111111111"),
          BigInt.fromU32(1),
          Address.fromString("0x4444444444444444444444444444444444444444")
        )
      )
    );

    createTradeEntity(
      event,
      orderHash,
      inputVaultBalanceChange,
      outputVaultBalanceChange
    );

    let id = makeTradeId(event, orderHash).toHexString();
    let orderId = makeOrderId(event.address, orderHash).toHexString();

    assert.entityCount("Trade", 1);
    assert.fieldEquals(
      "Trade",
      id,
      "timestamp",
      event.block.timestamp.toString()
    );

    assert.fieldEquals(
      "Trade",
      id,
      "inputVaultBalanceChange",
      inputVaultBalanceChange.id.toHexString()
    );
    assert.fieldEquals(
      "Trade",
      id,
      "outputVaultBalanceChange",
      outputVaultBalanceChange.id.toHexString()
    );
    assert.fieldEquals("Trade", id, "order", orderId);
  });
});
