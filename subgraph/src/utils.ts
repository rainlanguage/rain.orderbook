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
  OrderBook,
  OrderClear,
  RainMetaV1,
  SignedContext,
  TakeOrderEntity,
  TokenVault,
  Transaction,
  Vault,
  VaultDeposit,
  VaultWithdraw,
} from "../generated/schema";
import { ReserveToken } from "../generated/OrderBook/ReserveToken";
import { ClearAliceStruct } from "../generated/OrderBook/OrderBook";

export const RAIN_META_DOCUMENT_HEX = "0xff0a89c674ee7874";

// Orderbook: TakeOrder(address sender, TakeOrderConfig config, uint256 input, uint256 output)
export let TAKE_ORDER_EVENT_TOPIC =
  "0x219a030b7ae56e7bea2baab709a4a45dc174a1f85e57730e5cb395bc32962542";

// Orderbook: Clear(address sender, Order alice, Order bob, ClearConfig clearConfig)
export let CLEAR_EVENT_TOPIC =
  "0xd153812deb929a6e4378f6f8cf61d010470840bf2e736f43fb2275803958bfa2";

// Orderbook: AfterClear(address sender, ClearStateChange clearStateChange);
export let AFTER_CLEAR_EVENT_TOPIC =
  "0x3f20e55919cca701abb2a40ab72542b25ea7eed63a50f979dd2cd3231e5f488d";

// ExpressionDeployer: NewExpression(address sender, bytes[] sources, uint256[] constants, uint256[] minOutputs)
export let NEW_EXPRESSION_EVENT_TOPIC =
  "0xf66a0c19428b142e06d7aa23d5f18b9b9ff08408fefcdfb8bb27cb34929f7786";

export const tuplePrefix = Bytes.fromHexString(
  "0000000000000000000000000000000000000000000000000000000000000020"
);

/**
 * From a given hexadecimal string, check if it's have an even length
 */
export function getEvenHex(value_: string): string {
  if (value_.length % 2) {
    value_ = value_.slice(0, 2) + "0" + value_.slice(2);
  }

  return value_;
}

export function stringToArrayBuffer(val: string): ArrayBuffer {
  const buff = new ArrayBuffer(val.length / 2);
  const view = new DataView(buff);
  for (let i = 0, j = 0; i < val.length; i = i + 2, j++) {
    view.setUint8(j, u8(Number.parseInt(`${val.at(i)}${val.at(i + 1)}`, 16)));
  }
  return buff;
}

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
    token.decimals = !decimals.reverted ? decimals.value : 0;
    token.name = !name.reverted ? name.value : "NONE";
    token.symbol = !symbol.reverted ? symbol.value : "NONE";
    token.totalSupply = BigInt.zero();
    token.totalSupplyDisplay = BigDecimal.zero();
    token.save();
  }
  let totalSupply = reserveToken.try_totalSupply();
  token.totalSupply = !totalSupply.reverted
    ? totalSupply.value
    : token.totalSupply;
  token.save();
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
  let orderHashHex = getEvenHex(keccak256.toHex());

  let order_ = Order.load(orderHashHex);
  if (order_) return order_;
  else return new Order(orderHashHex);
}

function hexToBI(hexString: string): BigInt {
  return BigInt.fromUnsignedBytes(
    changetype<Bytes>(Bytes.fromHexString(hexString).reverse())
  );
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

export function getOB(obAddress_: Address): OrderBook {
  let orderBook = OrderBook.load(obAddress_);
  if (!orderBook) {
    orderBook = new OrderBook(obAddress_);
    orderBook.address = obAddress_;
    orderBook.save();
  }
  return orderBook;
}

export function getRainMetaV1(meta_: Bytes): RainMetaV1 {
  const metaV1_ID = getKeccak256FromBytes(meta_);

  let metaV1 = RainMetaV1.load(metaV1_ID);

  if (!metaV1) {
    metaV1 = new RainMetaV1(metaV1_ID);
    metaV1.metaBytes = meta_;
    metaV1.save();
  }

  return metaV1;
}

export function getKeccak256FromBytes(data_: Bytes): Bytes {
  return Bytes.fromByteArray(crypto.keccak256(Bytes.fromByteArray(data_)));
}

export function isHexadecimalString(str: string): boolean {
  // Check if string is empty
  if (str.length == 0) {
    return false;
  }

  // Check if each character is a valid hexadecimal character
  for (let i = 0; i < str.length; i++) {
    let charCode = str.charCodeAt(i);
    if (
      !(
        (charCode >= 48 && charCode <= 57) || // 0-9
        (charCode >= 65 && charCode <= 70) || // A-F
        (charCode >= 97 && charCode <= 102)
      )
    ) {
      // a-f
      return false;
    }
  }

  return true;
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

export function toDisplay(amount: BigInt, token: string): BigDecimal {
  let erc20 = createToken(Address.fromString(token));
  if (erc20) {
    let denominator = BigInt.fromString(getZeros(erc20.decimals));
    return amount.toBigDecimal().div(denominator.toBigDecimal());
  }
  return amount.toBigDecimal().div(BigDecimal.fromString(getZeros(0)));
}

function getZeros(num: number): string {
  let s = "1";
  for (let i = 0; i < num; i++) {
    s = s + "0";
  }
  return s;
}

export function gcd(a: BigInt, b: BigInt): BigInt {
  if (b.equals(BigInt.zero())) {
    return a;
  } else {
    return gcd(b, a.mod(b));
  }
}

export function BDtoBIMultiplier(n1: BigDecimal, n2: BigDecimal): BigInt {
  let n1_split = n1.toString().split(".");
  let n1_decimals = n1_split.length == 1 ? 0 : n1_split[1].length;

  let n2_split = n2.toString().split(".");
  let n2_decimals = n2_split.length == 1 ? 0 : n2_split[1].length;

  let number: BigDecimal;
  if (n1_decimals > n2_decimals) {
    number = n1;
  } else {
    number = n2;
  }
  let location = number.toString().indexOf(".");
  let len = number.toString().slice(location + 1).length;
  return BigInt.fromString(getZeros(len));
}

export function createSignedContext(
  txHash: string,
  logIndex: string
): SignedContext {
  for (let i = 0; ; i++) {
    let signedContext = SignedContext.load(`${txHash}-${logIndex}-${i}`);
    if (!signedContext) {
      return new SignedContext(`${txHash}-${logIndex}-${i}`);
    }
  }
  return new SignedContext("");
}
export function createContextEntity(
  txHash: string,
  logIndex: string
): ContextEntity {
  for (let i = 0; ; i++) {
    let contextEntity = ContextEntity.load(`${txHash}-${logIndex}-${i}`);
    if (!contextEntity) {
      return new ContextEntity(`${txHash}-${logIndex}-${i}`);
    }
  }
  return new ContextEntity("");
}
