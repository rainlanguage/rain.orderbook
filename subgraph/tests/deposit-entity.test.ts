import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address, crypto, Bytes } from "@graphprotocol/graph-ts";
import { createDepositEntity } from "../src/deposit";
import { createDepositEvent } from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";
import { createMockERC20Functions } from "./erc20.test";
import { FLOAT_100, FLOAT_ZERO } from "./float.test";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createDepositEntity()", () => {
    createMockERC20Functions(
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    let vaultId = Bytes.fromHexString(
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );

    let sender = Address.fromString(
      "0x1234567890123456789012345678901234567890"
    );
    let token = Address.fromString(
      "0x0987654321098765432109876543210987654321"
    );

    let event = createDepositEvent(sender, token, vaultId, BigInt.fromI32(100));
    createDepositEntity(event, FLOAT_ZERO, FLOAT_100, FLOAT_100);

    let id = crypto.keccak256(
      event.address.concat(
        event.transaction.hash.concatI32(event.logIndex.toI32())
      )
    );
    let vaultEId = vaultEntityId(event.address, sender, vaultId, token);

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
      vaultEId.toHexString()
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
