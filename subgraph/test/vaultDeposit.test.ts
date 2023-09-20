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
import { DepositEvent } from "../typechain/contracts/orderbook/OrderBook";
import {
  deployOBMultiTx,
  getEventArgs,
  waitForSubgraphToBeSynced,
} from "./utils";
import { DepositConfigStruct } from "../typechain/contracts/orderbook/OrderBook";

describe("VaultDeposit entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    await tokenA.initialize();
    await tokenB.initialize();
  });

  it("should query the VaultDeposit after a deposit", async function () {
    const [, alice] = signers;

    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const amount = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenA.transfer(alice.address, amount);

    // Deposit config using different tokens
    const depositConfigStruct: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: aliceOutputVault,
      amount: amount,
    };

    await tokenA
      .connect(alice)
      .approve(orderBook.address, depositConfigStruct.amount);

    // Alice deposits both tokens into her output vault
    const txDeposit = await orderBook
      .connect(alice)
      .deposit(depositConfigStruct);

    const { sender: depositSender, config: depositConfig } =
      (await getEventArgs(
        txDeposit,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    // Checking Config A
    assert(depositSender === alice.address);
    compareStructs(depositConfig, depositConfigStruct);

    // Subgrpah check
    await waitForSubgraphToBeSynced();

    // VaultDeposit ID: `tx.hash-{N}` where n is the N deposit with the same tx.hash;
    // In this case, the tx only made one deposit, so the N is 0
    const vaultDeposit_ID = `${txDeposit.hash.toLowerCase()}-0`;

    // Vault ID: #{vaultId}-{owner}
    const vault_ID = `${depositConfig.vaultId.toString()}-${alice.address.toLowerCase()}`;

    // TokenVault ID: #{vaultId}-{owner}-{token}
    const tokenVault_A_ID = `${depositConfig.vaultId.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;

    const query = `{
        vaultDeposit (id: "${vaultDeposit_ID}") {
          amount
          vaultId
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

    const data = response.data.vaultDeposit;

    assert.equal(data.amount, depositConfig.amount.toString());
    assert.equal(data.vaultId, depositConfig.vaultId.toString());
    assert.equal(data.sender.id, depositSender.toLowerCase());
    assert.equal(data.token.id, tokenA.address.toLowerCase());
    assert.equal(data.vault.id, vault_ID);
    assert.equal(data.tokenVault.id, tokenVault_A_ID);
  });

  it("should query correctly the VaultDeposits after deposits on same transactions", async function () {
    const [, alice] = signers;

    // Deploy new OBMultiTx contract
    const obMultiTx = await deployOBMultiTx(orderBook);

    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const amountA = ethers.BigNumber.from("1000" + eighteenZeros);
    const amountB = ethers.BigNumber.from("2000" + eighteenZeros);

    await tokenA.transfer(alice.address, amountA);
    await tokenB.transfer(alice.address, amountB);

    // Deposit config using different tokens and amounts
    const depositConfigStruct_A: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: aliceOutputVault,
      amount: amountA,
    };
    const depositConfigStruct_B: DepositConfigStruct = {
      token: tokenB.address,
      vaultId: aliceOutputVault,
      amount: amountB,
    };

    // Approve the tokens to be used by the OBMultiTx contract
    await tokenA
      .connect(alice)
      .approve(obMultiTx.address, depositConfigStruct_A.amount);
    await tokenB
      .connect(alice)
      .approve(obMultiTx.address, depositConfigStruct_B.amount);

    // Alice deposits both tokens using the obMultiTx
    const txMultiDeposits = await obMultiTx
      .connect(alice)
      .multiDeposit([depositConfigStruct_A, depositConfigStruct_B]);

    const depositEvents = (await getEvents(
      txMultiDeposits,
      "Deposit",
      orderBook
    )) as Array<DepositEvent["args"]>;

    // Subgraph check
    await waitForSubgraphToBeSynced();

    for (let i = 0; i < depositEvents.length; i++) {
      // Using the Deposit event for a given index
      const { sender, config } = depositEvents[i];

      // VaultDeposit ID: `tx.hash-{N}` where n is the N deposit with the same tx.hash;
      // In this case, the N value is the i value given the iteration
      const vaultDeposit_ID = `${txMultiDeposits.hash.toLowerCase()}-${i}`;

      // Vault ID = {Deposit.config.vaultId}-{sender}
      const vault_ID = `${config.vaultId.toString()}-${sender.toLowerCase()}`;

      // TokenVault ID = {Deposit.config.vaultId}-{Deposit.sender}-{Deposit.config.token}
      const tokenVault_ID = `${config.vaultId.toString()}-${sender.toLowerCase()}-${config.token.toLowerCase()}`;

      const query = `{
        vaultDeposit (id: "${vaultDeposit_ID}") {
          vaultId
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

      const data = response.data.vaultDeposit;

      assert.equal(data.vaultId, config.vaultId.toString());
      assert.equal(data.amount, config.amount.toString());
      assert.equal(data.sender.id, sender.toLowerCase());
      assert.equal(data.token.id, config.token.toLowerCase());
      assert.equal(data.vault.id, vault_ID);
      assert.equal(data.tokenVault.id, tokenVault_ID);
    }
  });
});
