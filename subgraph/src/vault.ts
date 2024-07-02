import { Bytes, BigInt } from "@graphprotocol/graph-ts";
import { Vault } from "../generated/schema";
import { getERC20Entity } from "./erc20";

export function vaultEntityId(
  owner: Bytes,
  vaultId: BigInt,
  token: Bytes
): Bytes {
  return owner.concat(
    token.concat(Bytes.fromByteArray(Bytes.fromBigInt(vaultId)))
  );
}

export function createEmptyVault(
  owner: Bytes,
  vaultId: BigInt,
  token: Bytes
): Vault {
  let vault = new Vault(vaultEntityId(owner, vaultId, token));
  vault.vaultId = vaultId;
  vault.token = getERC20Entity(token);
  vault.owner = owner;
  vault.balance = BigInt.fromI32(0);
  vault.save();
  return vault;
}

export function getVault(owner: Bytes, vaultId: BigInt, token: Bytes): Vault {
  let vault = Vault.load(vaultEntityId(owner, vaultId, token));
  if (vault == null) {
    vault = createEmptyVault(owner, vaultId, token);
  }
  return vault;
}

export function handleVaultBalanceChange(
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes
): BigInt {
  let vault = getVault(owner, vaultId, token);
  let oldVaultBalance = vault.balance;
  vault.balance = vault.balance.plus(amount);
  vault.save();
  return oldVaultBalance;
}
