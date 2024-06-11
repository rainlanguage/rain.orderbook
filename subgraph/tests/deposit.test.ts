import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  newMockEvent,
  clearInBlockStore,
} from "matchstick-as";
import { BigInt, ethereum, Address } from "@graphprotocol/graph-ts";
import { Deposit } from "../generated/Deposit/OrderBook";
import { createDepositEntity } from "../src/deposit";

describe("Deposits", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createDepositEntity()", () => {
    let event = createDepositEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0xabcdefABCDEFabcdefABCDEFabcdefABCDEFabcdef"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    createDepositEntity(event);

    let id = event.transaction.hash.toHex() + "-" + event.logIndex.toString();

    assert.entityCount("Deposit", 1);
    assert.fieldEquals("Deposit", id, "amount", BigInt.fromI32(100).toString());
    assert.fieldEquals(
      "Deposit",
      id,
      "sender",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals("Deposit", id, "vaultId", BigInt.fromI32(1).toString());
    assert.fieldEquals(
      "Deposit",
      id,
      "token",
      "0xabcdefABCDEFabcdefABCDEFabcdefABCDEFabcdef"
    );
    assert.fieldEquals(
      "Deposit",
      id,
      "transaction",
      event.transaction.hash.toHex()
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

// event Deposit(address sender, address token, uint256 vaultId, uint256 amount);
function createDepositEvent(
  sender: Address,
  token: Address,
  vaultId: BigInt,
  amount: BigInt
): Deposit {
  let mockEvent = newMockEvent();
  let depositEvent = new Deposit(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  depositEvent.parameters = new Array();
  depositEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  depositEvent.parameters.push(
    new ethereum.EventParam("token", ethereum.Value.fromAddress(token))
  );
  depositEvent.parameters.push(
    new ethereum.EventParam(
      "vaultId",
      ethereum.Value.fromUnsignedBigInt(vaultId)
    )
  );
  depositEvent.parameters.push(
    new ethereum.EventParam("amount", ethereum.Value.fromUnsignedBigInt(amount))
  );
  return depositEvent;
}
