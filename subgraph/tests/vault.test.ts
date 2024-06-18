import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  newMockEvent,
  clearInBlockStore,
} from "matchstick-as";
import { createDepositEvent } from "./deposit.test";
import { createWithdrawalEvent } from "./withdrawal.test";
import { handleVaultBalanceChange } from "../src/vault";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleVaultBalanceChange()", () => {
    handleVaultBalanceChange(
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      0
    );

    let vaultEntityId = Bytes.fromHexString(
      "0x1234567890123456789012345678901234567890"
    ).concatI32(1);

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultDeposit()", () => {
    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender,
      0
    );

    let vaultEntityId = event.params.token.concatI32(
      event.params.vaultId.toI32()
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultWithdraw()", () => {
    // first we need to deposit
    let depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200)
    );

    handleVaultBalanceChange(
      depositEvent.params.vaultId,
      depositEvent.params.token,
      depositEvent.params.amount,
      depositEvent.params.sender,
      0
    );

    // then we withdraw
    let event = createWithdrawalEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender,
      1
    );

    let vaultEntityId = event.params.token.concatI32(
      event.params.vaultId.toI32()
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultEntityId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });
});
