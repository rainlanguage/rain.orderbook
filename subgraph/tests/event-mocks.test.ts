import { newMockEvent } from "matchstick-as";
import { BigInt, ethereum, Address } from "@graphprotocol/graph-ts";
import { Deposit } from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";

// event Deposit(address sender, address token, uint256 vaultId, uint256 amount);
export function createDepositEvent(
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

// event Withdraw(address sender, address token, uint256 vaultId, uint256 targetAmount, uint256 amount);
export function createWithdrawEvent(
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
