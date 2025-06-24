import { Address, BigInt } from "@graphprotocol/graph-ts";
import { Deposit as DepositEntity } from "../generated/schema";
import { eventId } from "./interfaces/event";
import { handleVaultBalanceChange, vaultEntityId } from "./vault";
import { DepositV2 } from "../generated/OrderBook/OrderBook";
import { Float, getCalculator } from "./float";
import { ERC20 } from "../generated/OrderBook/ERC20";

export function handleDeposit(event: DepositV2): void {
  let erc20 = ERC20.bind(Address.fromBytes(event.params.token));
  let decimals = erc20.decimals();

  const calculator = getCalculator();
  let depositAmount = calculator.fromFixedDecimalLosslessPacked(
    event.params.depositAmountUint256,
    decimals
  );

  let vaultBalanceChange = handleVaultBalanceChange(
    event.address,
    event.params.vaultId,
    event.params.token,
    depositAmount,
    event.params.sender
  );

  let oldVaultBalance = vaultBalanceChange.oldVaultBalance;
  let newVaultBalance = vaultBalanceChange.newVaultBalance;

  createDepositEntity(event, oldVaultBalance, newVaultBalance, depositAmount);
}

export function createDepositEntity(
  event: DepositV2,
  oldVaultBalance: Float,
  newVaultBalance: Float,
  depositAmount: Float
): void {
  let deposit = new DepositEntity(eventId(event));
  deposit.orderbook = event.address;
  deposit.amount = depositAmount;
  deposit.sender = event.params.sender;
  deposit.vault = vaultEntityId(
    event.address,
    event.params.sender,
    event.params.vaultId,
    event.params.token
  );
  deposit.transaction = event.transaction.hash;
  deposit.oldVaultBalance = oldVaultBalance;
  deposit.newVaultBalance = newVaultBalance;
  deposit.timestamp = event.block.timestamp;
  deposit.save();
}
