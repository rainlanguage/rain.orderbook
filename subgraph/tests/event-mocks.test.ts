import { newMockEvent } from "matchstick-as";
import {
  BigInt,
  ethereum,
  Address,
  Bytes,
  Value,
} from "@graphprotocol/graph-ts";
import {
  AddOrderV2,
  ClearV2,
  AfterClear,
  Deposit,
  MetaV1_2,
  RemoveOrderV2,
  TakeOrderV2,
} from "../generated/OrderBook/OrderBook";
import { Withdraw } from "../generated/OrderBook/OrderBook";
import { createTransactionEntity } from "../src/transaction";

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

  createTransactionEntity(depositEvent);
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

  createTransactionEntity(withdrawalEvent);
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

export function createOrder(
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

// event TakeOrderV2(address sender, TakeOrderConfigV3 config, uint256 input, uint256 output);
export function createTakeOrderEvent(
  sender: Address,
  owner: Address,
  validInputs: Array<IO>,
  validOutputs: Array<IO>,
  nonce: Bytes,
  evaluable: Evaluable,
  input: BigInt,
  output: BigInt
): TakeOrderV2 {
  let mockEvent = newMockEvent();
  let takeOrderEvent = new TakeOrderV2(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  takeOrderEvent.parameters = new Array();

  takeOrderEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );

  let config = new ethereum.Tuple();

  let signedContext = new ethereum.Tuple();
  signedContext.push(ethereum.Value.fromAddress(sender));
  signedContext.push(ethereum.Value.fromUnsignedBigIntArray([]));
  signedContext.push(ethereum.Value.fromBytes(Bytes.fromHexString("0x")));

  let signedContextArray = new Array<ethereum.Tuple>();
  signedContextArray.push(signedContext);

  let order = createOrder(owner, evaluable, validInputs, validOutputs, nonce);

  config.push(ethereum.Value.fromTuple(order));
  config.push(ethereum.Value.fromUnsignedBigInt(BigInt.fromI32(0)));
  config.push(ethereum.Value.fromUnsignedBigInt(BigInt.fromI32(0)));
  config.push(ethereum.Value.fromTupleArray(signedContextArray));

  takeOrderEvent.parameters.push(
    new ethereum.EventParam("config", ethereum.Value.fromTuple(config))
  );
  takeOrderEvent.parameters.push(
    new ethereum.EventParam("input", ethereum.Value.fromUnsignedBigInt(input))
  );
  takeOrderEvent.parameters.push(
    new ethereum.EventParam("output", ethereum.Value.fromUnsignedBigInt(output))
  );
  return takeOrderEvent;
}

// event MetaV1(address sender, uint256 subject, bytes meta);
export function createMetaEvent(
  sender: Address,
  subject: Bytes,
  meta: Bytes
): MetaV1_2 {
  let mockEvent = newMockEvent();
  let metaEvent = new MetaV1_2(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  metaEvent.parameters = new Array();
  metaEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  metaEvent.parameters.push(
    new ethereum.EventParam("subject", ethereum.Value.fromBytes(subject))
  );
  metaEvent.parameters.push(
    new ethereum.EventParam("meta", ethereum.Value.fromBytes(meta))
  );
  return metaEvent;
}

// event ClearV2(address,(address,(address,address,bytes),(address,uint8,uint256)[],(address,uint8,uint256)[],bytes32),(address,(address,address,bytes),(address,uint8,uint256)[],(address,uint8,uint256)[],bytes32),(uint256,uint256,uint256,uint256,uint256,uint256))
export function createClearEvent(
  sender: Address,
  aliceOrder: ethereum.Tuple,
  bobOrder: ethereum.Tuple,
  aliceInputIOIndex: BigInt,
  aliceOutputIOIndex: BigInt,
  bobInputIOIndex: BigInt,
  bobOutputIOIndex: BigInt,
  aliceBountyVaultId: BigInt,
  bobBountyVaultId: BigInt
): ClearV2 {
  let mockEvent = newMockEvent();
  let clearEvent = new ClearV2(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );

  clearEvent.parameters = new Array();
  clearEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  clearEvent.parameters.push(
    new ethereum.EventParam("alice", ethereum.Value.fromTuple(aliceOrder))
  );
  clearEvent.parameters.push(
    new ethereum.EventParam("bob", ethereum.Value.fromTuple(bobOrder))
  );

  let _clearConfig = new ethereum.Tuple();
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(aliceInputIOIndex));
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(aliceOutputIOIndex));
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(bobInputIOIndex));
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(bobOutputIOIndex));
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(aliceBountyVaultId));
  _clearConfig.push(ethereum.Value.fromUnsignedBigInt(bobBountyVaultId));
  clearEvent.parameters.push(
    new ethereum.EventParam(
      "clearConfig",
      ethereum.Value.fromTuple(_clearConfig)
    )
  );

  return clearEvent;
}

// event AfterClear(address,(uint256,uint256,uint256,uint256))
export function createAfterClearEvent(
  sender: Address,
  aliceOutput: BigInt,
  bobOutput: BigInt,
  aliceInput: BigInt,
  bobInput: BigInt
): AfterClear {
  let mockEvent = newMockEvent();
  let afterClearEvent = new AfterClear(
    mockEvent.address,
    mockEvent.logIndex,
    mockEvent.transactionLogIndex,
    mockEvent.logType,
    mockEvent.block,
    mockEvent.transaction,
    mockEvent.parameters,
    null
  );
  afterClearEvent.parameters = new Array();
  afterClearEvent.parameters.push(
    new ethereum.EventParam("sender", ethereum.Value.fromAddress(sender))
  );
  let _clearStateChange = new ethereum.Tuple();
  _clearStateChange.push(ethereum.Value.fromUnsignedBigInt(aliceOutput));
  _clearStateChange.push(ethereum.Value.fromUnsignedBigInt(bobOutput));
  _clearStateChange.push(ethereum.Value.fromUnsignedBigInt(aliceInput));
  _clearStateChange.push(ethereum.Value.fromUnsignedBigInt(bobInput));
  afterClearEvent.parameters.push(
    new ethereum.EventParam(
      "clearStateChange",
      ethereum.Value.fromTuple(_clearStateChange)
    )
  );
  return afterClearEvent;
}
