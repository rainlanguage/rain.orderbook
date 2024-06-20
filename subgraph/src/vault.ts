import { Bytes, BigInt } from "@graphprotocol/graph-ts";
import { Withdraw, Deposit } from "../generated/OrderBook/OrderBook";
import { Vault } from "../generated/schema";
import { createDepositEntity } from "./deposit";
import { createWithdrawalEntity } from "./withdraw";
import { eventId } from "./interfaces/event";

export function vaultEntityId(vaultId: BigInt, token: Bytes): Bytes {
  return token.concat(Bytes.fromByteArray(Bytes.fromBigInt(vaultId)));
}

export function handleVaultBalanceChange(
  vaultId: BigInt,
  token: Bytes,
  amount: BigInt,
  owner: Bytes,
  direction: VaultBalanceChangeType
): BigInt {
  let oldVaultBalance: BigInt;
  let vault = Vault.load(vaultEntityId(vaultId, token));
  if (vault == null) {
    vault = new Vault(vaultEntityId(vaultId, token));
    vault.vaultId = vaultId;
    vault.token = token;
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

export function handleVaultDeposit(event: Deposit): void {
  let oldVaultBalance: BigInt = handleVaultBalanceChange(
    event.params.vaultId,
    event.params.token,
    event.params.amount,
    event.params.sender,
    VaultBalanceChangeType.CREDIT
  );
  createDepositEntity(event, oldVaultBalance);
}

export function handleVaultWithdraw(event: Withdraw): void {
  let oldVaultBalance: BigInt = handleVaultBalanceChange(
    event.params.vaultId,
    event.params.token,
    event.params.amount,
    event.params.sender,
    VaultBalanceChangeType.DEBIT
  );
  createWithdrawalEntity(event, oldVaultBalance);
}

enum VaultBalanceChangeType {
  CREDIT,
  DEBIT,
}
