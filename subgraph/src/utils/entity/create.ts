import {
  Bytes,
  BigInt,
  Address,
  ethereum,
  crypto,
  BigDecimal,
} from "@graphprotocol/graph-ts";
import {
  Account,
  ContextEntity,
  ERC20,
  Order,
  OrderClear,
  SignedContext,
  TakeOrderEntity,
  TokenVault,
  Transaction,
  Vault,
  VaultDeposit,
  VaultWithdraw,
} from "../../../generated/schema";
import { ReserveToken } from "../../../generated/OrderBook/ReserveToken";
import { ClearAliceStruct } from "../../../generated/OrderBook/OrderBook";
import {
  getEvenHexString,
  toDisplayWithDecimals,
} from "@rainprotocol/subgraph-utils";

export function createAccount(address: Bytes): Account {
  let account = Account.load(address);
  if (!account) {
    account = new Account(address);
    account.save();
  }
  return account;
}

export function createToken(address: Bytes): ERC20 {
  let token = ERC20.load(address.toHex());
  let reserveToken = ReserveToken.bind(Address.fromBytes(address));
  if (!token) {
    token = new ERC20(address.toHex());

    let decimals = reserveToken.try_decimals();
    let name = reserveToken.try_name();
    let symbol = reserveToken.try_symbol();
    let totalSupply = reserveToken.try_totalSupply();

    token.decimals = !decimals.reverted ? decimals.value : 0;
    token.name = !name.reverted ? name.value : "NONE";
    token.symbol = !symbol.reverted ? symbol.value : "NONE";
    token.totalSupply = !totalSupply.reverted
      ? totalSupply.value
      : BigInt.zero();

    if (!totalSupply.reverted && !decimals.reverted) {
      token.totalSupplyDisplay = toDisplayWithDecimals(
        totalSupply.value,
        decimals.value
      );
    } else {
      token.totalSupplyDisplay = BigDecimal.zero();
    }

    token.save();
  }
  // else {
  // let totalSupply = reserveToken.try_totalSupply();
  // if (!totalSupply.reverted) {
  //   let value = totalSupply.value;
  //   token.totalSupply = value;
  //   token.totalSupplyDisplay = toDisplayWithDecimals(value, token.decimals);
  // }
  // }

  return token;
}

export function createVault(vaultId: string, owner: Bytes): Vault {
  let vault = Vault.load(`${vaultId}-${owner.toHex()}`);
  if (!vault) {
    vault = new Vault(`${vaultId}-${owner.toHex()}`);
    vault.owner = createAccount(owner).id;

    vault.vaultId = BigInt.fromString(vaultId);
    vault.save();
  }
  return vault;
}

export function createTokenVault(
  vaultId: string,
  owner: Bytes,
  token: Bytes
): TokenVault {
  let tokenVault = TokenVault.load(
    `${vaultId}-${owner.toHex()}-${token.toHex()}`
  );
  if (!tokenVault) {
    tokenVault = new TokenVault(`${vaultId}-${owner.toHex()}-${token.toHex()}`);
    tokenVault.owner = createAccount(owner).id;
    tokenVault.token = createToken(token).id;
    tokenVault.balance = BigInt.zero();
    tokenVault.balanceDisplay = BigDecimal.zero();
    tokenVault.vault = createVault(vaultId, owner).id;
    tokenVault.vaultId = BigInt.fromString(vaultId);
    tokenVault.orders = [];
    tokenVault.orderClears = [];
    tokenVault.save();
  }
  return tokenVault;
}

export function createOrder(order: ClearAliceStruct): Order {
  let tupleEvaluable: Array<ethereum.Value> = [
    ethereum.Value.fromAddress(order.evaluable.interpreter),
    ethereum.Value.fromAddress(order.evaluable.store),
    ethereum.Value.fromAddress(order.evaluable.expression),
  ];

  let evaluable = changetype<ethereum.Tuple>(tupleEvaluable);

  let tupleValidInputs: Array<ethereum.Tuple> = [];
  for (let i = 0; i < order.validInputs.length; i++) {
    let VI: Array<ethereum.Value> = [
      ethereum.Value.fromAddress(order.validInputs[i].token),
      ethereum.Value.fromI32(order.validInputs[i].decimals),
      ethereum.Value.fromUnsignedBigInt(order.validInputs[i].vaultId),
    ];

    tupleValidInputs.push(changetype<ethereum.Tuple>(VI));
  }

  let tupleValidOutputs: Array<ethereum.Tuple> = [];
  for (let i = 0; i < order.validOutputs.length; i++) {
    let VO: Array<ethereum.Value> = [
      ethereum.Value.fromAddress(order.validOutputs[i].token),
      ethereum.Value.fromI32(order.validOutputs[i].decimals),
      ethereum.Value.fromUnsignedBigInt(order.validOutputs[i].vaultId),
    ];

    tupleValidOutputs.push(changetype<ethereum.Tuple>(VO));
  }

  let tupleArray: Array<ethereum.Value> = [
    ethereum.Value.fromAddress(order.owner),
    ethereum.Value.fromBoolean(order.handleIO),
    ethereum.Value.fromTuple(evaluable),
    ethereum.Value.fromTupleArray(tupleValidInputs),
    ethereum.Value.fromTupleArray(tupleValidOutputs),
  ];

  let tuple = changetype<ethereum.Tuple>(tupleArray);
  let encodedOrder = ethereum.encode(ethereum.Value.fromTuple(tuple))!;
  let keccak256 = crypto.keccak256(encodedOrder);
  let orderHashHex = getEvenHexString(keccak256.toHex());

  let order_loaded = Order.load(orderHashHex);
  if (order_loaded) return order_loaded;
  else return new Order(orderHashHex);
}

export function createTakeOrderConfig(txHash: string): TakeOrderEntity {
  for (let i = 0; ; i++) {
    let orderClear = TakeOrderEntity.load(`${txHash}-${i}`);
    if (!orderClear) {
      return new TakeOrderEntity(`${txHash}-${i}`);
    }
  }
  return new TakeOrderEntity("");
}

export function createOrderClear(txHash: string): OrderClear {
  for (let i = 0; ; i++) {
    let orderClear = OrderClear.load(`${txHash}-${i}`);
    if (!orderClear) {
      return new OrderClear(`${txHash}-${i}`);
    }
  }
  return new OrderClear("");
}

export function createVaultDeposit(txHash: string): VaultDeposit {
  for (let i = 0; ; i++) {
    let orderClear = VaultDeposit.load(`${txHash}-${i}`);
    if (!orderClear) {
      return new VaultDeposit(`${txHash}-${i}`);
    }
  }
  return new VaultDeposit("");
}

export function createVaultWithdraw(txHash: string): VaultWithdraw {
  for (let i = 0; ; i++) {
    let orderClear = VaultWithdraw.load(`${txHash}-${i}`);
    if (!orderClear) {
      return new VaultWithdraw(`${txHash}-${i}`);
    }
  }
  return new VaultWithdraw("");
}

export function createTransaction(
  hash: string,
  block: ethereum.Block
): Transaction {
  let transaction = Transaction.load(hash);
  if (!transaction) {
    transaction = new Transaction(hash);
    transaction.blockNumber = block.number;
    transaction.timestamp = block.timestamp;
    transaction.save();
  }
  return transaction;
}

export function createSignedContext(txHash: string): SignedContext {
  for (let i = 0; ; i++) {
    let signedContext = SignedContext.load(`${txHash}-${i}`);
    if (!signedContext) {
      return new SignedContext(`${txHash}-${i}`);
    }
  }
  return new SignedContext("");
}
export function createContextEntity(txHash: string): ContextEntity {
  for (let i = 0; ; i++) {
    let contextEntity = ContextEntity.load(`${txHash}-${i}`);
    if (!contextEntity) {
      return new ContextEntity(`${txHash}-${i}`);
    }
  }
  return new ContextEntity("");
}
