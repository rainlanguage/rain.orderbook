import { Bytes, BigInt, crypto } from "@graphprotocol/graph-ts";
import { Vault } from "../generated/schema";
import { getERC20Entity } from "./erc20";

export function vaultEntityId(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: BigInt,
  token: Bytes
): Bytes {
  let bytes = orderbook.concat(
    owner.concat(token.concat(Bytes.fromByteArray(Bytes.fromBigInt(vaultId))))
  );
  return Bytes.fromByteArray(crypto.keccak256(bytes));
}

export function createEmptyVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: BigInt,
  token: Bytes
): Vault {
  let vault = new Vault(vaultEntityId(orderbook, owner, vaultId, token));
  vault.orderbook = orderbook;
  vault.vaultId = vaultId;
  vault.token = getERC20Entity(token);
  vault.owner = owner;
  vault.balance = BigInt.fromI32(0);
  vault.totalVolumeIn = BigInt.fromI32(0);
  vault.totalVolumeOut = BigInt.fromI32(0);
  vault.save();
  return vault;
}

export function getVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: BigInt,
  token: Bytes
): Vault {
  let vault = Vault.load(vaultEntityId(orderbook, owner, vaultId, token));
  if (vault == null) {
    vault = createEmptyVault(orderbook, owner, vaultId, token);
  }
  return vault;
}

export function handleVaultBalanceChange(
  orderbook: Bytes,
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes
): BigInt {
  let vault = getVault(orderbook, owner, vaultId, token);
  let oldVaultBalance = vault.balance;
  vault.balance = vault.balance.plus(amount);
  vault.save();
  return oldVaultBalance;
}

export function handleTradeVaultBalanceChange(
  orderbook: Bytes,
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes
): BigInt {
  let oldVaultBalance = handleVaultBalanceChange(
    orderbook,
    vaultId,
    token,
    amount,
    owner
  );
  let vault = getVault(orderbook, owner, vaultId, token);
  if (amount.lt(BigInt.fromI32(0))) {
    vault.totalVolumeOut = vault.totalVolumeOut.plus(amount.neg());
  } else {
    vault.totalVolumeIn = vault.totalVolumeIn.plus(amount);
  }
  vault.save();
  return oldVaultBalance;
}
