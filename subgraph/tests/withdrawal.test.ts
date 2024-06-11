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
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { createWithdrawalEntity } from "../src/withdraw";

describe("Withdrawals", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createWithdrawalEntity()", () => {
    let event = createWithdrawalEvent(
      Address.fromString("0x1234567890123456789012345678901234567890"),
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(100)
    );
    createWithdrawalEntity(event);

    let id = event.transaction.hash.toHex() + "-" + event.logIndex.toString();

    assert.entityCount("Withdrawal", 1);
    assert.fieldEquals(
      "Withdrawal",
      id,
      "amount",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id,
      "targetAmount",
      BigInt.fromI32(200).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id,
      "sender",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Withdrawal",
      id,
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Withdrawal",
      id,
      "token",
      "0x0987654321098765432109876543210987654321"
    );
    assert.fieldEquals(
      "Withdrawal",
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
  });
});

// event Withdraw(address sender, address token, uint256 vaultId, uint256 targetAmount, uint256 amount);
function createWithdrawalEvent(
  sender: Address,
  token: Address,
  vaultId: BigInt,
  targetAmount: BigInt,
  amount: BigInt
): Withdraw {
  let mockEvent = newMockEvent();
  let withdrawalEvent = new Withdraw(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  withdrawalEvent.parameters = new Array();
  withdrawalEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  withdrawalEvent.parameters.push(
    new ethereum.EventParam("token", ethereum.Value.fromAddress(token))
  );
  withdrawalEvent.parameters.push(
    new ethereum.EventParam(
      "vaultId",
      ethereum.Value.fromUnsignedBigInt(vaultId)
    )
  );
  withdrawalEvent.parameters.push(
    new ethereum.EventParam(
      "targetAmount",
      ethereum.Value.fromUnsignedBigInt(targetAmount)
    )
  );
  withdrawalEvent.parameters.push(
    new ethereum.EventParam("amount", ethereum.Value.fromUnsignedBigInt(amount))
  );
  return withdrawalEvent;
}
