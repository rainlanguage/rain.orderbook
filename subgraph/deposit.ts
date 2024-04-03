import { Deposit } from "../generated/OrderBook/OrderBook";
import {
  createAccount,
  createToken,
  createTokenVault,
  createTransaction,
  createVault,
  createVaultDeposit,
  toDisplay,
} from "./utils";

export function handleDeposit(event: Deposit): void {
  let tokenVault = createTokenVault(
    event.params.vaultId.toString(),
    event.params.sender,
    event.params.token,
  );

  if (tokenVault) {
    tokenVault.balance = tokenVault.balance.plus(event.params.amount);
    tokenVault.balanceDisplay = toDisplay(
      tokenVault.balance,
      event.params.token.toHexString(),
    );
    tokenVault.save();
  }

  let vaultDeposit = createVaultDeposit(event.transaction.hash.toHex());
  vaultDeposit.sender = createAccount(event.params.sender).id;
  vaultDeposit.token = createToken(event.params.token).id;
  vaultDeposit.vaultId = event.params.vaultId;
  vaultDeposit.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender,
  ).id;
  vaultDeposit.amount = event.params.amount;
  vaultDeposit.amountDisplay = toDisplay(
    vaultDeposit.amount,
    event.params.token.toHexString(),
  );
  vaultDeposit.tokenVault = tokenVault.id;
  vaultDeposit.vault = createVault(
    event.params.vaultId.toString(),
    event.params.sender,
  ).id;
  vaultDeposit.transaction = createTransaction(
    event.transaction.hash.toHex(),
    event.block,
  ).id;
  vaultDeposit.emitter = createAccount(event.params.sender).id;
  vaultDeposit.timestamp = event.block.timestamp;
  vaultDeposit.save();
}
