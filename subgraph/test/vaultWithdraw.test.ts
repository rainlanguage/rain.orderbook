import assert from "assert";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { ReserveToken18 } from "../typechain";
import {
  basicDeploy,
  compareStructs,
  eighteenZeros,
  getEvents,
  randomUint256,
} from "../utils";
import { ethers } from "hardhat";
import {
  DepositEvent,
  WithdrawConfigStruct,
  WithdrawEvent,
} from "../typechain/contracts/orderbook/OrderBook";
import {
  deployOBMultiTx,
  getEventArgs,
  waitForSubgraphToBeSynced,
} from "./utils";
import { DepositConfigStruct } from "../typechain/contracts/orderbook/OrderBook";

describe("VaultWithdraw entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    await tokenA.initialize();
    await tokenB.initialize();
  });

  it("should query the VaultWithdraw after withdrawal", async function () {
    const [, alice] = signers;

    const vaultId = ethers.BigNumber.from(randomUint256());

    // DEPOSIT
    const amount = ethers.BigNumber.from("1000" + eighteenZeros);
    await tokenA.transfer(alice.address, amount);

    const depositConfigStruct: DepositConfigStruct = {
      token: tokenA.address,
      vaultId,
      amount: amount,
    };

    await tokenA
      .connect(alice)
      .approve(orderBook.address, depositConfigStruct.amount);

    // Alice deposits tokenA into her non-append-only vault
    const txDeposit = await orderBook
      .connect(alice)
      .deposit(depositConfigStruct);

    const { sender: depositSender, config: depositConfig } =
      (await getEventArgs(
        txDeposit,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    assert(depositSender === alice.address);
    compareStructs(depositConfig, depositConfigStruct);

    const aliceTokenABalance0 = await tokenA.balanceOf(alice.address);

    const withdrawConfigStruct: WithdrawConfigStruct = {
      token: tokenA.address,
      vaultId: vaultId,
      amount: amount,
    };

    const txWithdraw = await orderBook
      .connect(alice)
      .withdraw(withdrawConfigStruct);

    const {
      sender: withdrawSender,
      config: withdrawConfig,
      amount: withdrawnAmount,
    } = (await getEventArgs(
      txWithdraw,
      "Withdraw",
      orderBook
    )) as WithdrawEvent["args"];

    assert(withdrawSender === alice.address);
    compareStructs(withdrawConfig, withdrawConfigStruct);

    const aliceTokenABalance1 = await tokenA.balanceOf(alice.address);

    assert(aliceTokenABalance0.isZero());
    assert(aliceTokenABalance1.eq(amount));

    // Checking the VaultIDs
    assert(
      depositConfig.vaultId.eq(withdrawConfig.vaultId),
      "Deposit and Withdraw does not have the same VaultID"
    );

    // Subgraph check
    await waitForSubgraphToBeSynced();

    // VaultWithdraw ID: `tx.hash-{N}` where n is the N withdraw with the same tx.hash;
    // In this case, the tx only made one withdraw, so the N is 0
    const vaultWithdraw_ID = `${txWithdraw.hash.toLowerCase()}-0`;

    // Vault ID: #{vaultId}-{owner}
    const vault_ID = `${withdrawConfig.vaultId.toString()}-${alice.address.toLowerCase()}`;

    // TokenVault ID: #{vaultId}-{owner}-{token}
    const tokenVault_ID = `${withdrawConfig.vaultId.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;

    const query = `{
      vaultWithdraw (id: "${vaultWithdraw_ID}") {
        vaultId
        requestedAmount
        amount
        sender {
          id
        }
        token {
          id
        }
        vault {
          id
        }
        tokenVault {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data = response.data.vaultWithdraw;

    assert.equal(data.vaultId, withdrawConfig.vaultId.toString());
    assert.equal(data.requestedAmount, withdrawConfig.amount.toString());
    assert.equal(data.amount, withdrawnAmount.toString());
    assert.equal(data.sender.id, withdrawSender.toLowerCase());
    assert.equal(data.token.id, withdrawConfig.token.toLowerCase());
    assert.equal(data.vault.id, vault_ID);
    assert.equal(data.tokenVault.id, tokenVault_ID);
  });

  it("should query correctly the VaultWithdraws after withdraws on same transactions", async function () {
    const [, alice] = signers;

    // Deploy new OBMultiTx contract
    const obMultiTx = await deployOBMultiTx(orderBook);

    const vaultId = ethers.BigNumber.from(randomUint256());

    // DEPOSIT
    const amountToDeposit = ethers.BigNumber.from("2000" + eighteenZeros);
    await tokenA.transfer(alice.address, amountToDeposit);

    // Make one deposit with enought amount
    const depositConfigStruct: DepositConfigStruct = {
      token: tokenA.address,
      vaultId,
      amount: amountToDeposit,
    };

    // Approve the OBMultiTx contract to use the alice tokens
    await tokenA
      .connect(alice)
      .approve(obMultiTx.address, depositConfigStruct.amount);

    // Make the deposit using the OBMultiTx contract (the owner will be the contract)
    const txDeposits = await obMultiTx
      .connect(alice)
      .multiDeposit([depositConfigStruct]);

    const { config: depositConfig } = (await getEventArgs(
      txDeposits,
      "Deposit",
      orderBook
    )) as DepositEvent["args"];

    // assert(depositSender === alice.address);
    compareStructs(depositConfig, depositConfigStruct);

    // WITHDRAWS
    // First amount to be requested for withdrawal will be 1/4 of the deposited amount.
    const amountToWithdraw_A = amountToDeposit.div(4);

    // Second amount to be requested for withdrawal will be 3/4 of the amount deposited.
    const amountToWithdraw_B = amountToDeposit.div(4).mul(3);

    const withdrawConfigStruct_A: WithdrawConfigStruct = {
      token: tokenA.address,
      vaultId: vaultId,
      amount: amountToWithdraw_A,
    };

    const withdrawConfigStruct_B: WithdrawConfigStruct = {
      token: tokenA.address,
      vaultId: vaultId,
      amount: amountToWithdraw_B,
    };

    // Alice use the contract to multi withdraws (Contract will hold the tokens)
    const txMultiWithdraws = await obMultiTx
      .connect(alice)
      .multiWithdraw([withdrawConfigStruct_A, withdrawConfigStruct_B]);

    const withdrawEvents = (await getEvents(
      txMultiWithdraws,
      "Withdraw",
      orderBook
    )) as Array<WithdrawEvent["args"]>;

    // Subgraph check
    await waitForSubgraphToBeSynced();

    //
    for (let i = 0; i < withdrawEvents.length; i++) {
      // Using the Withdraw event for a given index
      const {
        sender: withdrawSender,
        config: withdrawConfig,
        amount: withdrawnAmount,
      } = withdrawEvents[i];

      // VaultWithdraw ID: `tx.hash-{N}` where n is the N withdraw with the same tx.hash;
      // In this case, the N value is the i value given the iteration
      const vaultWithdraw_ID = `${txMultiWithdraws.hash.toLowerCase()}-${i}`;

      // Vault ID = {Withdraw.config.vaultId}-{sender}
      const vault_ID = `${withdrawConfig.vaultId.toString()}-${withdrawSender.toLowerCase()}`;

      // TokenVault ID = {Withdraw.config.vaultId}-{Withdraw.sender}-{Withdraw.config.token}
      const tokenVault_ID = `${withdrawConfig.vaultId.toString()}-${withdrawSender.toLowerCase()}-${withdrawConfig.token.toLowerCase()}`;

      const query = `{
        vaultWithdraw (id: "${vaultWithdraw_ID}") {
          vaultId
          requestedAmount
          amount
          sender {
            id
          }
          token {
            id
          }
          vault {
            id
          }
          tokenVault {
            id
          }
        }
      }`;

      const response = (await subgraph({ query })) as FetchResult;

      const data = response.data.vaultWithdraw;

      assert.equal(data.vaultId, withdrawConfig.vaultId.toString());
      assert.equal(data.requestedAmount, withdrawConfig.amount.toString());
      assert.equal(data.amount, withdrawnAmount.toString());
      assert.equal(data.sender.id, withdrawSender.toLowerCase());
      assert.equal(data.token.id, withdrawConfig.token.toLowerCase());
      assert.equal(data.vault.id, vault_ID);
      assert.equal(data.tokenVault.id, tokenVault_ID);
    }
  });
});
