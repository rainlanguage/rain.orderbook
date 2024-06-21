import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEvent } from "../event-mocks.test";
import { handleDeposit } from "../../src/handlers";
import { vaultEntityId } from "../../src/vault";
import { Deposit, Vault } from "../../generated/schema";
import { eventId } from "../../src/interfaces/event";

describe("Handle deposit", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleDeposit()", () => {
    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );

    handleDeposit(event);

    // check vault entity
    let vault = Vault.load(
      vaultEntityId(
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );

    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(100));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check deposit entity
    let deposit = Deposit.load(eventId(event));

    assert.assertNotNull(deposit);
    if (deposit == null) {
      return;
    }
    assert.bytesEquals(deposit.sender, event.params.sender);
    assert.bytesEquals(deposit.token, event.params.token);
    assert.bigIntEquals(deposit.amount, BigInt.fromI32(100));
    assert.bigIntEquals(deposit.oldVaultBalance, BigInt.fromI32(0));
    assert.bigIntEquals(deposit.newVaultBalance, BigInt.fromI32(100));

    // make another deposit, same token, same vaultId
    event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200)
    );

    handleDeposit(event);

    // check vault entity
    vault = Vault.load(
      vaultEntityId(
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );
    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(300));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check deposit entity
    deposit = Deposit.load(eventId(event));

    assert.assertNotNull(deposit);
    if (deposit == null) {
      return;
    }
    assert.bytesEquals(deposit.sender, event.params.sender);
    assert.bytesEquals(deposit.token, event.params.token);
    assert.bigIntEquals(deposit.amount, BigInt.fromI32(200));
    assert.bigIntEquals(deposit.oldVaultBalance, BigInt.fromI32(100));
    assert.bigIntEquals(deposit.newVaultBalance, BigInt.fromI32(300));

    // make another deposit, different token, same vaultId
    event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(300)
    );

    handleDeposit(event);

    // check vault entity
    vault = Vault.load(
      vaultEntityId(
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );
    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(300));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check deposit entity
    deposit = Deposit.load(eventId(event));

    assert.assertNotNull(deposit);
    if (deposit == null) {
      return;
    }
    assert.bytesEquals(deposit.sender, event.params.sender);
    assert.bytesEquals(deposit.token, event.params.token);
    assert.bigIntEquals(deposit.amount, BigInt.fromI32(300));
    assert.bigIntEquals(deposit.oldVaultBalance, BigInt.fromI32(0));
    assert.bigIntEquals(deposit.newVaultBalance, BigInt.fromI32(300));
  });
});
