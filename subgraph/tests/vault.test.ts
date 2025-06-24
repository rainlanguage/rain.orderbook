import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { handleVaultBalanceChange, vaultEntityId } from "../src/vault";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "./event-mocks.test";
import { createMockERC20Functions } from "./erc20.test";
import { FLOAT_100, FLOAT_ZERO } from "./float.test";

describe("Vault balance changes", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleVaultBalanceChange()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let owner = "0x0987654321098765432109876543210987654321";

    let orderbook =
      "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    let vaultId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let token = "0x1234567890123456789012345678901234567890";

    handleVaultBalanceChange(
      Bytes.fromHexString(orderbook),
      Bytes.fromHexString(vaultId),
      Bytes.fromHexString(token),
      FLOAT_100,
      Bytes.fromHexString(owner)
    );

    let vaultEId = vaultEntityId(
      Bytes.fromHexString(orderbook),
      Bytes.fromHexString(owner),
      Bytes.fromHexString(vaultId),
      Bytes.fromHexString(token)
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEId.toHexString(),
      "balance",
      FLOAT_100.toString()
    );
    assert.fieldEquals("Vault", vaultEId.toHexString(), "token", token);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "vaultId", vaultId);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "owner", owner);
  });

  test("handleVaultDeposit()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      FLOAT_100,
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
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
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      BigInt.fromI32(200)
    );

    handleVaultBalanceChange(
      depositEvent.address,
      depositEvent.params.vaultId,
      depositEvent.params.token,
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000000c8"
      ),
      depositEvent.params.sender
    );

    // then we withdraw
    let event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000000c8"
      ),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      Bytes.fromHexString(
        "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff9c"
      ),
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
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
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      BigInt.fromI32(100)
    );

    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      event.params.sender
    );

    let vaultId = vaultEntityId(
      event.address,
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
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

    let balanceChange = handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.bytesEquals(balanceChange.oldVaultBalance, FLOAT_ZERO);
  });

  test("handleVaultBalanceChange returns old balance if vault exists", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    let balanceChange = handleVaultBalanceChange(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321")
    );

    assert.bytesEquals(
      balanceChange.oldVaultBalance,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      )
    );
  });
});
