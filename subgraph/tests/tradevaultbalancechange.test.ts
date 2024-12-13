import {
  test,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  newMockEvent,
  assert,
} from "matchstick-as";
import { BigInt, Address, Bytes, crypto } from "@graphprotocol/graph-ts";
import {
  Evaluable,
  IO,
  createAfterClearEvent,
  createTakeOrderEvent,
} from "./event-mocks.test";
import { eventId } from "../src/interfaces/event";
import {
  createTradeVaultBalanceChangeEntity,
  tradeVaultBalanceChangeId,
} from "../src/tradevaultbalancechange";
import { vaultEntityId } from "../src/vault";
import { orderHashFromTakeOrderEvent } from "../src/takeorder";
import { makeTradeId } from "../src/trade";
import { createMockERC20Functions } from "./erc20.test";
import { makeClearBountyId } from "../src/clear";

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

    let tradeVaultBalanceChangeId = Bytes.fromByteArray(
      crypto.keccak256(eventId(event).concat(vaultEntityId))
    );

    assert.bytesEquals(
      tradeVaultBalanceChangeId,
      Bytes.fromHexString(
        "0x417b9a4b4f93565a22e3d13b93f7f467b0e84ef8dddcb52a718298e4e17df26f"
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
      event.address,
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

  test("TradeVaultBalanceChangeEntity id should not be equal to ClearBounty id", () => {
    const alice = Address.fromString(
      "0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce"
    );
    let aliceOutputAmount = BigInt.fromString("10");
    let bobOutputAmount = BigInt.fromString("20");
    let aliceInputAmount = BigInt.fromString("15");
    let bobInputAmount = BigInt.fromString("10");
    let vaultEntityId = Bytes.fromHexString(
      "0x1234567890abcdef1234567890abcdef12345678"
    );
    let event = createAfterClearEvent(
      alice,
      aliceOutputAmount,
      bobOutputAmount,
      aliceInputAmount,
      bobInputAmount
    );
    const id1 = tradeVaultBalanceChangeId(event, vaultEntityId);
    const id2 = makeClearBountyId(event, vaultEntityId);

    assert.assertTrue(id1 !== id2);
  });
});
