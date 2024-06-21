import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  log,
} from "matchstick-as";
import { BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "../event-mocks.test";
import { handleDeposit, handleWithdraw } from "../../src/handlers";
import { vaultEntityId } from "../../src/vault";
import { Withdrawal, Vault } from "../../generated/schema";
import { eventId } from "../../src/interfaces/event";

describe("Handle withdraw", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleWithdraw()", () => {
    // first we make a deposit
    let depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(1000)
    );
    handleDeposit(depositEvent);

    // then we make a withdraw
    let event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(150),
      BigInt.fromI32(100)
    );

    handleWithdraw(event);

    // check vault entity
    let vault = Vault.load(
      vaultEntityId(event.params.vaultId, event.params.token)
    );

    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(900));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    let withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(withdraw.token, event.params.token);
    assert.bigIntEquals(withdraw.amount, BigInt.fromI32(100));
    assert.bigIntEquals(withdraw.oldVaultBalance, BigInt.fromI32(1000));
    assert.bigIntEquals(withdraw.newVaultBalance, BigInt.fromI32(900));

    // make another withdraw, same token, same vaultId
    event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(200)
    );

    handleWithdraw(event);

    // check vault entity
    vault = Vault.load(vaultEntityId(event.params.vaultId, event.params.token));

    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(700));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(withdraw.token, event.params.token);
    assert.bigIntEquals(withdraw.amount, BigInt.fromI32(200));
    assert.bigIntEquals(withdraw.oldVaultBalance, BigInt.fromI32(900));
    assert.bigIntEquals(withdraw.newVaultBalance, BigInt.fromI32(700));

    // deposit different token, same vaultId
    depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(300)
    );

    handleDeposit(depositEvent);

    // make a withdraw for the new token
    event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(300),
      BigInt.fromI32(200)
    );

    handleWithdraw(event);

    // check vault entity
    vault = Vault.load(vaultEntityId(event.params.vaultId, event.params.token));

    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bigIntEquals(vault.balance, BigInt.fromI32(100));
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bigIntEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(withdraw.token, event.params.token);
    assert.bigIntEquals(withdraw.amount, BigInt.fromI32(200));
    assert.bigIntEquals(withdraw.oldVaultBalance, BigInt.fromI32(300));
    assert.bigIntEquals(withdraw.newVaultBalance, BigInt.fromI32(100));
  });
});
