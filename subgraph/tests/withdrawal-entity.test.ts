import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address } from "@graphprotocol/graph-ts";
import { createWithdrawalEntity } from "../src/withdraw";
import { createWithdrawEvent } from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";

describe("Withdrawals", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createWithdrawalEntity()", () => {
    let event = createWithdrawEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(100)
    );

    let oldVaultBalance = BigInt.fromI32(300);
    createWithdrawalEntity(event, oldVaultBalance);

    let id = event.transaction.hash.concatI32(event.logIndex.toI32());
    let vaultId = vaultEntityId(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Withdrawal", 1);
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "amount",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "targetAmount",
      BigInt.fromI32(200).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "sender",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "vault",
      vaultId.toHexString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "token",
      "0x0987654321098765432109876543210987654321"
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "transaction",
      event.transaction.hash.toHex()
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "oldVaultBalance",
      BigInt.fromI32(300).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id.toHexString(),
      "newVaultBalance",
      BigInt.fromI32(200).toString()
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
  });
});
