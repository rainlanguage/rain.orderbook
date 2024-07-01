import {
  test,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  newMockEvent,
  assert,
} from "matchstick-as";
import { BigInt, Address, Bytes } from "@graphprotocol/graph-ts";
import { Evaluable, IO, createTakeOrderEvent } from "./event-mocks.test";
import { eventId } from "../src/interfaces/event";
import {
  createTradeVaultBalanceChangeEntity,
  tradeVaultBalanceChangeId,
} from "../src/tradevaultbalancechange";
import { vaultEntityId } from "../src/vault";
import { orderHashFromTakeOrderEvent } from "../src/takeorder";
import { makeTradeId } from "../src/trade";
import { createMockERC20Functions } from "./erc20.test";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("tradeVaultBalanceChangeId()", () => {
    let event = newMockEvent();
    event.logIndex = BigInt.fromI32(2);
    event.transaction.hash = Bytes.fromHexString(
      "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    );
    let vaultEntityId = Bytes.fromHexString(
      "0x1234567890abcdef1234567890abcdef12345678"
    );

    let tradeVaultBalanceChangeId = vaultEntityId.concat(eventId(event));

    assert.bytesEquals(
      tradeVaultBalanceChangeId,
      Bytes.fromHexString(
        "0x1234567890abcdef1234567890abcdef123456781234567890abcdef" +
          "1234567890abcdef1234567890abcdef1234567890abcdef" +
          "02000000"
      )
    );
  });

  test("createTradeVaultBalanceChangeEntity()", () => {
    createMockERC20Functions(
      Address.fromString("0x3333333333333333333333333333333333333333")
    );
    createMockERC20Functions(
      Address.fromString("0x4444444444444444444444444444444444444444")
    );

    let owner = Address.fromString(
      "0x1111111111111111111111111111111111111111"
    );
    let event = createTakeOrderEvent(
      owner,
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

    let oldVaultBalance = BigInt.fromI32(10);

    let _vaultEntityId = vaultEntityId(
      owner,
      BigInt.fromI32(1),
      Address.fromString("0x3333333333333333333333333333333333333333")
    );

    let orderHash = orderHashFromTakeOrderEvent(event);

    createTradeVaultBalanceChangeEntity(
      event,
      orderHash,
      _vaultEntityId,
      oldVaultBalance,
      BigInt.fromI32(1)
    );

    assert.entityCount("TradeVaultBalanceChange", 1);

    let id = tradeVaultBalanceChangeId(event, _vaultEntityId).toHexString();

    assert.fieldEquals("TradeVaultBalanceChange", id, "amount", "1");
    assert.fieldEquals("TradeVaultBalanceChange", id, "oldVaultBalance", "10");
    assert.fieldEquals("TradeVaultBalanceChange", id, "newVaultBalance", "11");
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      id,
      "vault",
      _vaultEntityId.toHexString()
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      id,
      "trade",
      makeTradeId(event, orderHash).toHexString()
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      id,
      "timestamp",
      event.block.timestamp.toString()
    );
    assert.fieldEquals(
      "TradeVaultBalanceChange",
      id,
      "transaction",
      event.transaction.hash.toHex()
    );
  });
});
