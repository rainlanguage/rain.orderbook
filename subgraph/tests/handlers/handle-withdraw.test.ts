import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, Address, Bytes } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "../event-mocks.test";
import { handleDeposit, handleWithdraw } from "../../src/handlers";
import { vaultEntityId } from "../../src/vault";
import { Withdrawal, Vault } from "../../generated/schema";
import { eventId } from "../../src/interfaces/event";
import { createMockERC20Functions } from "../erc20.test";

describe("Handle withdraw", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleWithdraw()", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    // first we make a deposit
    let depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      BigInt.fromI32(1000)
    );
    handleDeposit(depositEvent);

    // then we make a withdraw
    let event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000096"
      ),
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      ),
      BigInt.fromI32(100)
    );

    handleWithdraw(event);

    // we should have an orderbook entity
    assert.entityCount("Orderbook", 1);
    assert.fieldEquals(
      "Orderbook",
      event.address.toHexString(),
      "id",
      event.address.toHexString()
    );

    // check vault entity
    let vault = Vault.load(
      vaultEntityId(
        event.address,
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );
    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bytesEquals(
      vault.balance,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000384"
      )
    );
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bytesEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    let withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(
      withdraw.amount,
      Bytes.fromHexString(
        "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff9c"
      )
    );
    assert.bytesEquals(
      withdraw.oldVaultBalance,
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000003e8"
      )
    );
    assert.bytesEquals(
      withdraw.newVaultBalance,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000384"
      )
    );

    // make another withdraw, same token, same vaultId
    event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000000c8"
      ),
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000000c8"
      ),
      BigInt.fromI32(200)
    );

    handleWithdraw(event);

    // check vault entity
    vault = Vault.load(
      vaultEntityId(
        event.address,
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );
    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bytesEquals(
      vault.balance,
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000002bc"
      )
    );
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bytesEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(
      withdraw.amount,
      Bytes.fromHexString(
        "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff38"
      )
    );
    assert.bytesEquals(
      withdraw.oldVaultBalance,
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000003e8"
      )
    );
    assert.bytesEquals(
      withdraw.newVaultBalance,
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000002bc"
      )
    );
    assert.bigIntEquals(withdraw.timestamp, event.block.timestamp);

    createMockERC20Functions(
      Address.fromString("0x0987654321098765432109876543210987654321")
    );

    // deposit different token, same vaultId
    depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      BigInt.fromI32(300)
    );

    handleDeposit(depositEvent);

    // make a withdraw for the new token
    event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Bytes.fromHexString(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      ),
      Bytes.fromHexString(
        "0x000000000000000000000000000000000000000000000000000000000000012c"
      ),
      Bytes.fromHexString(
        "0x00000000000000000000000000000000000000000000000000000000000000c8"
      ),
      BigInt.fromI32(200)
    );

    handleWithdraw(event);

    // check vault entity
    vault = Vault.load(
      vaultEntityId(
        event.address,
        event.params.sender,
        event.params.vaultId,
        event.params.token
      )
    );
    assert.assertNotNull(vault);
    if (vault == null) {
      return;
    }
    assert.bytesEquals(
      vault.balance,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      )
    );
    assert.bytesEquals(vault.owner, event.params.sender);
    assert.bytesEquals(vault.token, event.params.token);
    assert.bytesEquals(vault.vaultId, event.params.vaultId);

    // check withdraw entity
    withdraw = Withdrawal.load(eventId(event));

    assert.assertNotNull(withdraw);
    if (withdraw == null) {
      return;
    }
    assert.bytesEquals(withdraw.sender, event.params.sender);
    assert.bytesEquals(
      withdraw.amount,
      Bytes.fromHexString(
        "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff38"
      )
    );
    assert.bytesEquals(
      withdraw.oldVaultBalance,
      Bytes.fromHexString(
        "0x000000000000000000000000000000000000000000000000000000000000012c"
      )
    );
    assert.bytesEquals(
      withdraw.newVaultBalance,
      Bytes.fromHexString(
        "0x0000000000000000000000000000000000000000000000000000000000000064"
      )
    );
    assert.bigIntEquals(withdraw.timestamp, event.block.timestamp);
  });
});
