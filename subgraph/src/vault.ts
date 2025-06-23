import { Bytes, crypto } from "@graphprotocol/graph-ts";
import { Vault } from "../generated/schema";
import { getERC20Entity } from "./erc20";
import { Float, getCalculator } from "./float";

export type VaultId = Bytes;

export function vaultEntityId(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: VaultId,
  token: Bytes
): Bytes {
  let bytes = orderbook.concat(owner.concat(token.concat(vaultId)));
  return Bytes.fromByteArray(crypto.keccak256(bytes));
}

export function createEmptyVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: VaultId,
  token: Bytes
): Vault {
  let vault = new Vault(vaultEntityId(orderbook, owner, vaultId, token));
  vault.orderbook = orderbook;
  vault.vaultId = vaultId;
  vault.token = getERC20Entity(token);
  vault.owner = owner;
  vault.balance = Bytes.fromI32(0);
  vault.save();
  return vault;
}

export function getVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: Bytes,
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
  vaultId: Bytes,
  token: Bytes,
  amount: Float,
  owner: Bytes
): Float {
  let calculator = getCalculator();

  let vault = getVault(orderbook, owner, vaultId, token);
  let oldVaultBalance = vault.balance;
  vault.balance = calculator.add(oldVaultBalance, amount);
  vault.save();

  return oldVaultBalance;
}
