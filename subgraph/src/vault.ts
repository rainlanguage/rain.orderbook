import { Bytes, BigInt } from "@graphprotocol/graph-ts";
import { Withdraw, Deposit } from "../generated/OrderBook/OrderBook";
import { Vault } from "../generated/schema";

export function vaultEntityId(vaultId: BigInt, token: Bytes): Bytes {
  return token.concatI32(vaultId.toI32());
}

export function handleVaultBalanceChange(
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes,
  direction: VaultBalanceChangeType
): void {
  let vault = Vault.load(vaultEntityId(vaultId, token));
  if (vault == null) {
    vault = new Vault(vaultEntityId(vaultId, token));
    vault.vaultId = vaultId;
    vault.token = token;
    vault.owner = owner;
    vault.balance = BigInt.fromI32(0);
  }
  if (direction == VaultBalanceChangeType.CREDIT) {
    vault.balance = vault.balance.plus(amount);
  }
  if (direction == VaultBalanceChangeType.DEBIT) {
    vault.balance = vault.balance.minus(amount);
  }
  vault.save();
}

export function handleVaultDeposit(event: Deposit): void {
  handleVaultBalanceChange(
    event.params.vaultId,
    event.params.token,
    event.params.amount,
    event.params.sender,
    VaultBalanceChangeType.CREDIT
  );
}

export function handleVaultWithdraw(event: Withdraw): void {
  handleVaultBalanceChange(
    event.params.vaultId,
    event.params.token,
    event.params.amount,
    event.params.sender,
    VaultBalanceChangeType.DEBIT
  );
}

enum VaultBalanceChangeType {
  CREDIT,
  DEBIT,
}
