import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address, crypto, Bytes } from "@graphprotocol/graph-ts";
import { createWithdrawalEntity } from "../src/withdraw";
import { createWithdrawEvent } from "./event-mocks.test";
import { vaultEntityId } from "../src/vault";
import { createMockERC20Functions } from "./erc20.test";
import {
  createMockDecimalFloatFunctions,
  FLOAT_1,
  FLOAT_200,
  FLOAT_300,
} from "./float.test";
import { getCalculator } from "../src/float";

describe("Withdrawals", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createWithdrawalEntity()", () => {
    createMockDecimalFloatFunctions();
    createMockERC20Functions(
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    let event = createWithdrawEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      FLOAT_1,
      FLOAT_200,
      BigInt.fromI32(100)
    );

    let calculator = getCalculator();

    let oldVaultBalance = FLOAT_300;
    createWithdrawalEntity(calculator, event, oldVaultBalance);

    let id = crypto.keccak256(
      event.address.concat(
        event.transaction.hash.concatI32(event.logIndex.toI32())
      )
    );
    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Withdrawal", 1);
    assert.fieldEquals("Withdrawal", id.toHexString(), "amount", "-100");
    assert.fieldEquals("Withdrawal", id.toHexString(), "targetAmount", "200");
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
    assert.fieldEquals(
      "Withdrawal",
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
  });
});
