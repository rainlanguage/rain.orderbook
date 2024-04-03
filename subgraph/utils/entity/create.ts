import {
  Bytes,
  BigInt,
  Address,
  ethereum,
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
  getKeccak256FromBytes,
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
        decimals.value,
      );
    } else {
      token.totalSupplyDisplay = BigDecimal.zero();
    }

    token.save();
  }

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
  token: Bytes,
): TokenVault {
  let tokenVault = TokenVault.load(
    `${vaultId}-${owner.toHex()}-${token.toHex()}`,
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

export function createVaultDeposit(txHash: string): VaultDeposit {
  for (let i = 0; ; i++) {
    let orderClear = VaultDeposit.load(`${txHash}-${i}`);
    if (!orderClear) {
      return new VaultDeposit(`${txHash}-${i}`);
    }
  }
  return new VaultDeposit("");
}

export function createTransaction(
  hash: string,
  block: ethereum.Block,
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
