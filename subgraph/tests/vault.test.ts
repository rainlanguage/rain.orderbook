import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import {
  vaultEntityId,
  handleVaultBalanceChange,
  handleTradeVaultBalanceChange,
} from "../src/vault";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "./event-mocks.test";
import { createMockERC20Functions } from "./erc20.test";

describe("Vault balance changes", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleVaultBalanceChange()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    let vaultId = vaultEntityId(
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultDeposit()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultWithdraw()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    // first we need to deposit
    let depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200)
    );

    handleVaultBalanceChange(
      depositEvent.address,
      depositEvent.params.vaultId,
      depositEvent.params.token,
      depositEvent.params.amount,
      depositEvent.params.sender
    );

    // then we withdraw
    let event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      event.params.amount.neg(),
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("If vault does not exist, create it", () => {
    assert.entityCount("Vault", 0);

    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );

    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });
  test("handleVaultBalanceChange returns 0 if vault doesn't exist yet", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let oldBalance = handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.bigIntEquals(oldBalance, BigInt.fromI32(0));
  });

  test("handleVaultBalanceChange returns old balance if vault exists", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    let oldBalance = handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.bigIntEquals(oldBalance, BigInt.fromI32(100));
  });

  test("handleTradeVaultBalanceChange()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let vaultId = vaultEntityId(
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    handleTradeVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "totalVolumeIn",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "totalVolumeOut",
      BigInt.fromI32(0).toString()
    );

    handleTradeVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(-50),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(50).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "totalVolumeIn",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "totalVolumeOut",
      BigInt.fromI32(50).toString()
    );
  });
});
