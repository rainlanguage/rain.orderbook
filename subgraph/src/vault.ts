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

export function handleVaultBalanceChange(
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes,
  direction: VaultBalanceChangeType
): BigInt {
  let oldVaultBalance: BigInt;
  let vault = Vault.load(vaultEntityId(owner, vaultId, token));
  if (vault == null) {
    vault = new Vault(vaultEntityId(owner, vaultId, token));
    vault.vaultId = vaultId;
    vault.token = getERC20Entity(token);
    vault.owner = owner;
    vault.balance = BigInt.fromI32(0);
  }
  oldVaultBalance = vault.balance;
  if (direction == VaultBalanceChangeType.CREDIT) {
    vault.balance = vault.balance.plus(amount);
  }
  if (direction == VaultBalanceChangeType.DEBIT) {
    vault.balance = vault.balance.minus(amount);
  }
  vault.save();
  return oldVaultBalance;
}

export enum VaultBalanceChangeType {
  CREDIT,
  DEBIT,
}
