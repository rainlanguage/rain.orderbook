import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  beforeEach,
  createMockedFunction,
  dataSourceMock,
} from "matchstick-as";
import {
  createEmptyVault,
  handleVaultBalanceChange,
  handleVaultlessBalance,
  MUTLICALL3_ADDRESS,
  vaultEntityId,
  ZERO_BYTES_32
} from "../src/vault";
import { Bytes, BigInt, Address, ethereum } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "./event-mocks.test";
import { createMockERC20Functions } from "./erc20.test";
import {
  createMockDecimalFloatFunctions,
  FLOAT_100,
  FLOAT_200,
  FLOAT_NEG_100,
  FLOAT_0,
} from "./float.test";

describe("Vault balance changes", () => {
  beforeEach(createMockDecimalFloatFunctions);

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
      FLOAT_100.toHexString()
    );
    assert.fieldEquals("Vault", vaultEId.toHexString(), "token", token);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "vaultId", vaultId);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "owner", owner);
  });

  test("handleVaultDeposit()", () => {
    let token = "0x1234567890123456789012345678901234567890";
    createMockERC20Functions(Address.fromString(token));

    let sender = "0x0987654321098765432109876543210987654321";
    let vaultId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let event = createDepositEvent(
      Address.fromString(sender),
      Address.fromString(token),
      Bytes.fromHexString(vaultId),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      FLOAT_100,
      event.params.sender
    );

    let vaultEId = vaultEntityId(
      event.address,
      Address.fromString(sender),
      Bytes.fromHexString(vaultId),
      Address.fromString(token)
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEId.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
    assert.fieldEquals("Vault", vaultEId.toHexString(), "token", token);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "vaultId", vaultId);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "owner", sender);
  });

  test("handleVaultWithdraw()", () => {
    let token = "0x1234567890123456789012345678901234567890";
    createMockERC20Functions(Address.fromString(token));

    let sender = "0x0987654321098765432109876543210987654321";
    let vaultId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // first we need to deposit
    let depositEvent = createDepositEvent(
      Address.fromString(sender),
      Address.fromString(token),
      Bytes.fromHexString(vaultId),
      BigInt.fromI32(200)
    );

    handleVaultBalanceChange(
      depositEvent.address,
      depositEvent.params.vaultId,
      depositEvent.params.token,
      FLOAT_200,
      depositEvent.params.sender
    );

    // then we withdraw
    let event = createWithdrawEvent(
      Address.fromString(sender),
      Address.fromString(token),
      Bytes.fromHexString(vaultId),
      FLOAT_200,
      FLOAT_100,
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.address,
      event.params.vaultId,
      event.params.token,
      FLOAT_NEG_100,
      event.params.sender
    );

    let vaultEId = vaultEntityId(
      event.address,
      Address.fromString(sender),
      Bytes.fromHexString(vaultId),
      Address.fromString(token)
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultEId.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
    assert.fieldEquals("Vault", vaultEId.toHexString(), "token", token);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "vaultId", vaultId);
    assert.fieldEquals("Vault", vaultEId.toHexString(), "owner", sender);
  });

  test("If vault does not exist, create it", () => {
    assert.entityCount("Vault", 0);

    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let sender = "0x0987654321098765432109876543210987654321";
    let token = "0x1234567890123456789012345678901234567890";
    let vaultHexId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let event = createDepositEvent(
      Address.fromString(sender),
      Address.fromString(token),
      Bytes.fromHexString(vaultHexId),
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
      Address.fromString(sender),
      Bytes.fromHexString(vaultHexId),
      Address.fromString(token)
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
    assert.fieldEquals("Vault", vaultId.toHexString(), "token", token);
    assert.fieldEquals("Vault", vaultId.toHexString(), "vaultId", vaultHexId);
    assert.fieldEquals("Vault", vaultId.toHexString(), "owner", sender);
  });

  test("handleVaultBalanceChange returns 0 if vault doesn't exist yet", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let sender = "0x0987654321098765432109876543210987654321";
    let token = "0x1234567890123456789012345678901234567890";
    let vaultHexId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let balanceChange = handleVaultBalanceChange(
      Address.fromString(sender),
      Bytes.fromHexString(vaultHexId),
      Bytes.fromHexString(token),
      FLOAT_100,
      Bytes.fromHexString(sender)
    );

    assert.bytesEquals(balanceChange.oldVaultBalance, FLOAT_0);
  });

  test("handleVaultBalanceChange returns old balance if vault exists", () => {
    createMockERC20Functions(
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    let sender = "0x0987654321098765432109876543210987654321";
    let token = "0x1234567890123456789012345678901234567890";
    let vaultHexId =
      "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    handleVaultBalanceChange(
      Address.fromString(sender),
      Bytes.fromHexString(vaultHexId),
      Bytes.fromHexString(token),
      FLOAT_100,
      Bytes.fromHexString(sender)
    );

    let balanceChange = handleVaultBalanceChange(
      Address.fromString(sender),
      Bytes.fromHexString(vaultHexId),
      Bytes.fromHexString(token),
      FLOAT_100,
      Bytes.fromHexString(sender)
    );

    assert.bytesEquals(balanceChange.oldVaultBalance, FLOAT_100);
  });
});

describe("Vaultless balance updates", () => {
  beforeEach(createMockDecimalFloatFunctions);

  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleVaultlessBalance() updates balances for vaultless vaults", () => {
    let token1 = "0x1111111111111111111111111111111111111111";
    let token2 = "0x2222222222222222222222222222222222222222";
    createMockERC20Functions(Address.fromString(token1));
    createMockERC20Functions(Address.fromString(token2));

    let owner1 = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let owner2 = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let orderbook = Address.fromString("0xcccccccccccccccccccccccccccccccccccccccc");
    dataSourceMock.setAddress("0xcccccccccccccccccccccccccccccccccccccccc");

    // Create vaultless vaults (vaultId = 0x00...00)
    createEmptyVault(
      orderbook,
      Bytes.fromHexString(owner1),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token1),
    );
    createEmptyVault(
      orderbook,
      Bytes.fromHexString(owner2),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token2),
    );

    // mock multicall vaultBalance2() calls
    const call1 = changetype<ethereum.Tuple>([
      ethereum.Value.fromAddress(orderbook),
      ethereum.Value.fromBoolean(true),
      ethereum.Value.fromBytes(
        ethereum.encode(
          ethereum.Value.fromTuple(
            changetype<ethereum.Tuple>([
              ethereum.Value.fromAddress(Address.fromString(owner1)),
              ethereum.Value.fromAddress(Address.fromString(token1)),
              ethereum.Value.fromFixedBytes(Bytes.fromHexString(ZERO_BYTES_32))
            ])
          )
        )!
      )
    ]);
    const call2 = changetype<ethereum.Tuple>([
      ethereum.Value.fromAddress(orderbook),
      ethereum.Value.fromBoolean(true),
      ethereum.Value.fromBytes(
        ethereum.encode(
          ethereum.Value.fromTuple(
            changetype<ethereum.Tuple>([
              ethereum.Value.fromAddress(Address.fromString(owner2)),
              ethereum.Value.fromAddress(Address.fromString(token2)),
              ethereum.Value.fromFixedBytes(Bytes.fromHexString(ZERO_BYTES_32))
            ])
          )
        )!
      )
    ]);

    // Mock multicall3 aggregate3 response
    // Result format: ((bool success, bytes returnData)[])
    let result1 = changetype<ethereum.Tuple>([
      ethereum.Value.fromBoolean(true),
      ethereum.Value.fromBytes(FLOAT_100)
    ]);
    let result2 = changetype<ethereum.Tuple>([
      ethereum.Value.fromBoolean(true),
      ethereum.Value.fromBytes(FLOAT_200)
    ]);

    createMockedFunction(
      Address.fromString(MUTLICALL3_ADDRESS),
      "aggregate3",
      "aggregate3((address,bool,bytes)[]):((bool,bytes)[])"
    )
      .withArgs([ethereum.Value.fromTupleArray([
        call1,
        call2,
      ])])
      .returns([ethereum.Value.fromTupleArray([result1, result2])]);
    createMockedFunction(
      Address.fromString(MUTLICALL3_ADDRESS),
      "aggregate3",
      "aggregate3((address,bool,bytes)[]):((bool,bytes)[])"
    )
      .withArgs([ethereum.Value.fromTupleArray([
        call2,
        call1,
      ])])
      .returns([ethereum.Value.fromTupleArray([result2, result1])]);

    // Execute
    handleVaultlessBalance();

    // Verify balances were updated
    let vault1Id = vaultEntityId(
      orderbook,
      Bytes.fromHexString(owner1),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token1)
    );
    let vault2Id = vaultEntityId(
      orderbook,
      Bytes.fromHexString(owner2),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token2)
    );

    assert.fieldEquals(
      "Vault",
      vault1Id.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
    assert.fieldEquals(
      "Vault",
      vault2Id.toHexString(),
      "balance",
      FLOAT_200.toHexString()
    );
  });

  test("handleVaultlessBalance() ignores non-vaultless vaults", () => {
    let token = "0x1111111111111111111111111111111111111111";
    createMockERC20Functions(Address.fromString(token));

    let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let orderbook = Address.fromString("0xcccccccccccccccccccccccccccccccccccccccc");
    let nonZeroVaultId = "0x1111111111111111111111111111111111111111111111111111111111111111";
    dataSourceMock.setAddress("0xcccccccccccccccccccccccccccccccccccccccc");

    // Create a non-vaultless vault
    const vault =createEmptyVault(
      orderbook,
      Bytes.fromHexString(owner),
      Bytes.fromHexString(nonZeroVaultId),
      Bytes.fromHexString(token),
    );
    vault.balance = FLOAT_100;
    vault.save();

    // Mock empty multicall response since no vaultless vaults exist
    createMockedFunction(
      Address.fromString(MUTLICALL3_ADDRESS),
      "aggregate3",
      "aggregate3((address,bool,bytes)[]):((bool,bytes)[])"
    )
      .withArgs([ethereum.Value.fromTupleArray([])])
      .returns([ethereum.Value.fromTupleArray([])]);

    handleVaultlessBalance();

    // Balance should remain unchanged
    let vaultId = vaultEntityId(
      orderbook,
      Bytes.fromHexString(owner),
      Bytes.fromHexString(nonZeroVaultId),
      Bytes.fromHexString(token)
    );

    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
  });

  test("handleVaultlessBalance() handles multicall failures gracefully", () => {
    let token = "0x1111111111111111111111111111111111111111";
    createMockERC20Functions(Address.fromString(token));

    let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let orderbook = Address.fromString("0xcccccccccccccccccccccccccccccccccccccccc");
    dataSourceMock.setAddress("0xcccccccccccccccccccccccccccccccccccccccc");

    // Create vaultless vault
    const vault = createEmptyVault(
      orderbook,
      Bytes.fromHexString(owner),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token),
    );
    vault.balance = FLOAT_100;
    vault.save();

    // mock multicall vaultBalance2() calls
    const call = changetype<ethereum.Tuple>([
      ethereum.Value.fromAddress(orderbook),
      ethereum.Value.fromBoolean(true),
      ethereum.Value.fromBytes(
        ethereum.encode(
          ethereum.Value.fromTuple(
            changetype<ethereum.Tuple>([
              ethereum.Value.fromAddress(Address.fromString(owner)),
              ethereum.Value.fromAddress(Address.fromString(token)),
              ethereum.Value.fromFixedBytes(Bytes.fromHexString(ZERO_BYTES_32))
            ])
          )
        )!
      )
    ]);

    // Mock failed result
    let failedResult = changetype<ethereum.Tuple>([
      ethereum.Value.fromBoolean(false),
      ethereum.Value.fromBytes(Bytes.fromHexString("0x"))
    ]);

    createMockedFunction(
      Address.fromString(MUTLICALL3_ADDRESS),
      "aggregate3",
      "aggregate3((address,bool,bytes)[]):((bool,bytes)[])"
    )
      .withArgs([ethereum.Value.fromTupleArray([call])])
      .returns([ethereum.Value.fromTupleArray([failedResult])]);

    handleVaultlessBalance();

    // Balance should remain unchanged on failure
    let vaultId = vaultEntityId(
      orderbook,
      Bytes.fromHexString(owner),
      Bytes.fromHexString(ZERO_BYTES_32),
      Bytes.fromHexString(token)
    );

    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      FLOAT_100.toHexString()
    );
  });

  test("handleVaultlessBalance() handles empty vault list", () => {
    // No vaults created - should not error
    handleVaultlessBalance();
    
    assert.entityCount("Vault", 0);
  });
});
