import { assert, log, newMockEvent } from "matchstick-as";
import {
  BigInt,
  ethereum,
  Address,
  Bytes,
  Value,
} from "@graphprotocol/graph-ts";
import {
  AddOrderV2,
  Deposit,
  RemoveOrderV2,
} from "../generated/OrderBook/OrderBook";
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

export class IO {
  token: Address;
  decimals: BigInt;
  vaultId: BigInt;

  constructor(token: Address, decimals: BigInt, vaultId: BigInt) {
    this.token = token;
    this.decimals = decimals;
    this.vaultId = vaultId;
  }
}

export class Evaluable {
  interpreter: Address;
  store: Address;
  bytecode: Bytes;

  constructor(interpreter: Address, store: Address, bytecode: Bytes) {
    this.interpreter = interpreter;
    this.store = store;
    this.bytecode = bytecode;
  }
}

function createOrder(
  owner: Address,
  evaluable: Evaluable,
  validInputs: Array<IO>,
  validOutputs: Array<IO>,
  nonce: Bytes
): ethereum.Tuple {
  let _evaluable = new ethereum.Tuple();
  _evaluable.push(ethereum.Value.fromAddress(evaluable.interpreter));
  _evaluable.push(ethereum.Value.fromAddress(evaluable.store));
  _evaluable.push(ethereum.Value.fromBytes(evaluable.bytecode));

  let _validInputs = validInputs.map<ethereum.Tuple>((input) => {
    let _input = new ethereum.Tuple();
    _input.push(ethereum.Value.fromAddress(input.token));
    _input.push(ethereum.Value.fromUnsignedBigInt(input.decimals));
    _input.push(ethereum.Value.fromUnsignedBigInt(input.vaultId));
    return _input;
  });

  let _validOutputs = validOutputs.map<ethereum.Tuple>((output) => {
    let _output = new ethereum.Tuple();
    _output.push(ethereum.Value.fromAddress(output.token));
    _output.push(ethereum.Value.fromUnsignedBigInt(output.decimals));
    _output.push(ethereum.Value.fromUnsignedBigInt(output.vaultId));
    return _output;
  });

  let order = new ethereum.Tuple();
  order.push(ethereum.Value.fromAddress(owner));
  order.push(ethereum.Value.fromTuple(_evaluable));
  order.push(ethereum.Value.fromTupleArray(_validInputs));
  order.push(ethereum.Value.fromTupleArray(_validOutputs));
  order.push(ethereum.Value.fromFixedBytes(nonce));

  return order;
}

// event AddOrderV2(address sender, bytes32 orderHash, Order order);
export function createAddOrderEvent(
  sender: Address,
  orderHash: Bytes,
  validInputs: Array<IO>,
  validOutputs: Array<IO>,
  nonce: Bytes,
  evaluable: Evaluable
): AddOrderV2 {
  let mockEvent = newMockEvent();
  let addOrderEvent = new AddOrderV2(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  addOrderEvent.parameters = new Array();
  addOrderEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  addOrderEvent.parameters.push(
    new ethereum.EventParam(
      "orderHash",
      ethereum.Value.fromFixedBytes(orderHash)
    )
  );
  let order = createOrder(sender, evaluable, validInputs, validOutputs, nonce);
  addOrderEvent.parameters.push(
    new ethereum.EventParam("order", ethereum.Value.fromTuple(order))
  );

  return addOrderEvent;
}

// event RemoveOrderV2(address sender, bytes32 orderHash, OrderV3 order);
export function createRemoveOrderEvent(
  sender: Address,
  orderHash: Bytes,
  owner: Address,
  validInputs: Array<IO>,
  validOutputs: Array<IO>,
  nonce: Bytes,
  evaluable: Evaluable
): RemoveOrderV2 {
  let mockEvent = newMockEvent();
  let removeOrderEvent = new RemoveOrderV2(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  removeOrderEvent.parameters = new Array();
  removeOrderEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  removeOrderEvent.parameters.push(
    new ethereum.EventParam(
      "orderHash",
      ethereum.Value.fromFixedBytes(orderHash)
    )
  );
  let order = createOrder(owner, evaluable, validInputs, validOutputs, nonce);
  removeOrderEvent.parameters.push(
    new ethereum.EventParam("order", ethereum.Value.fromTuple(order))
  );
  return removeOrderEvent;
}
