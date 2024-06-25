import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEntity } from "../src/deposit";
import { createDepositEvent } from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";
import { createMockERC20Functions } from "./erc20.test";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createDepositEntity()", () => {
    createMockERC20Functions(
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    let event = createDepositEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    let newVaultBalance = BigInt.fromI32(0);
    createDepositEntity(event, newVaultBalance);

    let id = event.transaction.hash.concatI32(event.logIndex.toI32());
    let vaultId = vaultEntityId(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Deposit", 1);
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "amount",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "sender",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "vault",
      vaultId.toHexString()
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "transaction",
      event.transaction.hash.toHex()
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "oldVaultBalance",
      BigInt.fromI32(0).toString()
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "newVaultBalance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Deposit",
      id.toHexString(),
      "timestamp",
      event.block.timestamp.toString()
    );

    assert.entityCount("Transaction", 1);
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "blockNumber",
      event.block.number.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "timestamp",
      event.block.timestamp.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "from",
      event.transaction.from.toHex()
    );
  });
});
