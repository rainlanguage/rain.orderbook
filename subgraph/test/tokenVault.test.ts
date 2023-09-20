import assert from "assert";
import { expect } from "chai";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { ReserveToken18 } from "../typechain";
import {
  MemoryType,
  ONE,
  Opcode,
  basicDeploy,
  compareSolStructs,
  compareStructs,
  eighteenZeros,
  fixedPointDiv,
  fixedPointMul,
  generateEvaluableConfig,
  max_uint256,
  memoryOperand,
  minBN,
  op,
  randomUint256,
} from "../utils";
import { ethers } from "hardhat";
import { encodeMeta, getOrderConfig } from "../utils/orderBook/order";
import { concat } from "ethers/lib/utils";
import {
  AddOrderEvent,
  AfterClearEvent,
  ClearConfigStruct,
  ClearEvent,
  ClearStateChangeStruct,
  DepositConfigStruct,
  DepositEvent,
  OrderConfigStruct,
  WithdrawConfigStruct,
  WithdrawEvent,
} from "../typechain/contracts/orderbook/OrderBook";
import { getEventArgs, waitForSubgraphToBeSynced } from "./utils";

describe("TokenVault entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    await tokenA.initialize();
    await tokenB.initialize();
  });

  it("should query the TokenVault after adding an order", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);
    const constants_A = [max_uint256, ratio_A];
    const aOpMax = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 0)
    );
    const aRatio = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 1)
    );
    // prettier-ignore
    const source_A = concat([
      aOpMax,
      aRatio,
    ]);

    const EvaluableConfig_A = await generateEvaluableConfig(
      [source_A, []],
      constants_A
    );

    const orderConfig_A: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(orderConfig_A);

    const {
      sender: sender_A,
      expressionDeployer: ExpressionDeployer_A,
      order: order_A,
      orderHash,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(
      ExpressionDeployer_A === EvaluableConfig_A.deployer,
      "wrong expression deployer"
    );
    assert(sender_A === alice.address, "wrong sender");
    compareStructs(order_A, orderConfig_A);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    // TokenVault: #{vaultId}-{owner}-{token}
    const tokenVault_Input_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    const tokenVault_Output_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;

    // Vault: #{vaultId}-{owner}
    const vault_input_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}`;
    const vault_output_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}`;

    const query = `{
      tokenVaultInput: tokenVault (id: "${tokenVault_Input_ID}") {
        balance
        owner {
          id
        }
        vault {
          id
        }
        token {
          id
        }
        orders {
          id
        }
        orderClears {
          id
        }
      }
      tokenVaultOutput: tokenVault (id: "${tokenVault_Output_ID}") {
        balance
        owner {
          id
        }
        vault {
          id
        }
        token {
          id
        }
        orders {
          id
        }
        orderClears {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const dataInput = response.data.tokenVaultInput;
    const dataOutput = response.data.tokenVaultOutput;

    // TokenVaultInput
    assert.equal(dataInput.balance, 0);
    assert.equal(dataInput.owner.id, alice.address.toLowerCase());
    assert.equal(dataInput.vault.id, vault_input_ID);
    assert.equal(dataInput.token.id, tokenA.address.toLowerCase());
    expect(dataInput.orders).to.deep.include({
      id: orderHash.toHexString().toLowerCase(),
    });
    expect(dataInput.orderClears).to.be.empty;

    // TokenVaultOutput
    assert.equal(dataOutput.balance, 0);
    assert.equal(dataOutput.owner.id, alice.address.toLowerCase());
    assert.equal(dataOutput.vault.id, vault_output_ID);
    assert.equal(dataOutput.token.id, tokenB.address.toLowerCase());
    expect(dataOutput.orders).to.deep.include({
      id: orderHash.toHexString().toLowerCase(),
    });
    expect(dataOutput.orderClears).to.be.empty;
  });

  it("should update the TokenVault after adding orders with same Vault and Token", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    // Order
    const ratio = ethers.BigNumber.from("90" + eighteenZeros);
    const constants = [max_uint256, ratio];
    const OpMax = op(Opcode.read_memory, memoryOperand(MemoryType.Constant, 0));
    const Ratio = op(Opcode.read_memory, memoryOperand(MemoryType.Constant, 1));

    // prettier-ignore
    const source = concat([
      OpMax,
      Ratio,
    ]);

    const EvaluableConfig = await generateEvaluableConfig(
      [source, []],
      constants
    );

    const orderConfig: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(orderConfig);
    const txOrder_B = await orderBook.connect(alice).addOrder(orderConfig);

    const {
      sender: sender_A,
      expressionDeployer: ExpressionDeployer_A,
      order: order_A,
      orderHash: orderHash_A,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    const {
      sender: sender_B,
      expressionDeployer: ExpressionDeployer_B,
      order: order_B,
      orderHash: orderHash_B,
    } = (await getEventArgs(
      txOrder_B,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(
      ExpressionDeployer_A === EvaluableConfig.deployer,
      "wrong expression deployer"
    );
    assert(sender_A === alice.address, "wrong sender");
    compareStructs(order_A, orderConfig);

    assert(
      ExpressionDeployer_B === EvaluableConfig.deployer,
      "wrong expression deployer"
    );
    assert(sender_B === alice.address, "wrong sender");
    compareStructs(order_B, orderConfig);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    // TokenVault: #{vaultId}-{owner}-{token}
    const tokenVault_Input_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    const tokenVault_Output_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;

    const query = `{
      tokenVaultInput: tokenVault (id: "${tokenVault_Input_ID}") {
        orders {
          id
        }
      }
      tokenVaultOutput: tokenVault (id: "${tokenVault_Output_ID}") {
        orders {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const dataInput = response.data.tokenVaultInput;
    const dataOutput = response.data.tokenVaultOutput;

    // First order
    expect(dataInput.orders).to.deep.include({
      id: orderHash_A.toHexString().toLowerCase(),
    });
    expect(dataOutput.orders).to.deep.include({
      id: orderHash_B.toHexString().toLowerCase(),
    });

    // Second order
    expect(dataOutput.orders).to.deep.include({
      id: orderHash_A.toHexString().toLowerCase(),
    });
    expect(dataInput.orders).to.deep.include({
      id: orderHash_B.toHexString().toLowerCase(),
    });
  });

  it("should update the balance of the TokenVaults after deposits", async function () {
    const [, alice, bob] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const bobInputVault = ethers.BigNumber.from(randomUint256());
    const bobOutputVault = ethers.BigNumber.from(randomUint256());

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const OrderConfig_A: OrderConfigStruct = await getOrderConfig(
      ratio_A,
      max_uint256,
      tokenA.address,
      18,
      aliceInputVault,
      tokenB.address,
      18,
      aliceOutputVault,
      aliceOrder
    );

    const txAddOrderAlice = await orderBook
      .connect(alice)
      .addOrder(OrderConfig_A);

    const { sender: sender_A, order: Order_A } = (await getEventArgs(
      txAddOrderAlice,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_A === alice.address, "wrong sender");
    compareStructs(Order_A, OrderConfig_A);

    // Order_B
    const ratio_B = fixedPointDiv(ONE, ratio_A);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const bobOrder = encodeMeta("Order_B");

    const OrderConfig_B: OrderConfigStruct = await getOrderConfig(
      ratio_B,
      max_uint256,
      tokenB.address,
      18,
      bobInputVault,
      tokenA.address,
      18,
      bobOutputVault,
      bobOrder
    );

    const txAddOrderBob = await orderBook.connect(bob).addOrder(OrderConfig_B);

    const { sender: sender_B, order: Order_B } = (await getEventArgs(
      txAddOrderBob,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_B === bob.address, "wrong sender");
    compareStructs(Order_B, OrderConfig_B);

    // Checking SG entity after adding order to confirm status
    // Subgraph check
    await waitForSubgraphToBeSynced();

    // Alice Data IDs: (across the test)
    // - Token Vault:
    const tokenVault_Input_Alice_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    const tokenVault_Output_Alice_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;

    // Bob Data IDs:(across the test)
    // - Token Vault:
    const tokenVault_Input_Bob_ID = `${bobInputVault.toString()}-${bob.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;
    const tokenVault_Output_Bob_ID = `${bobOutputVault.toString()}-${bob.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;

    const query_0 = `{
      tokenVaultInputAlice: tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
      tokenVaultOutputAlice: tokenVault (id: "${tokenVault_Output_Alice_ID}") {
        balance
      }
      tokenVaultInputBob: tokenVault (id: "${tokenVault_Input_Bob_ID}") {
        balance
      }
      tokenVaultOutputBob: tokenVault (id: "${tokenVault_Output_Bob_ID}") {
        balance
      }
    }`;

    const response_0 = (await subgraph({ query: query_0 })) as FetchResult;

    const data_0 = response_0.data;

    // Alice check
    assert.equal(data_0.tokenVaultInputAlice.balance, 0);
    assert.equal(data_0.tokenVaultOutputAlice.balance, 0);

    // Bob check
    assert.equal(data_0.tokenVaultInputBob.balance, 0);
    assert.equal(data_0.tokenVaultOutputBob.balance, 0);

    // DEPOSITS
    const amountB = ethers.BigNumber.from("1000" + eighteenZeros);
    const amountA = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenB.transfer(alice.address, amountB);
    await tokenA.transfer(bob.address, amountA);

    const depositConfigStructAlice: DepositConfigStruct = {
      token: tokenB.address,
      vaultId: aliceOutputVault,
      amount: amountB,
    };
    const depositConfigStructBob: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: bobOutputVault,
      amount: amountA,
    };

    await tokenB
      .connect(alice)
      .approve(orderBook.address, depositConfigStructAlice.amount);
    await tokenA
      .connect(bob)
      .approve(orderBook.address, depositConfigStructBob.amount);

    // Alice deposits tokenB into her output vault
    const txDepositOrderAlice = await orderBook
      .connect(alice)
      .deposit(depositConfigStructAlice);
    // Bob deposits tokenA into his output vault
    const txDepositOrderBob = await orderBook
      .connect(bob)
      .deposit(depositConfigStructBob);

    const { sender: depositAliceSender, config: depositAliceConfig } =
      (await getEventArgs(
        txDepositOrderAlice,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];
    const { sender: depositBobSender, config: depositBobConfig } =
      (await getEventArgs(
        txDepositOrderBob,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    assert(depositAliceSender === alice.address);
    compareStructs(depositAliceConfig, depositConfigStructAlice);
    assert(depositBobSender === bob.address);
    compareStructs(depositBobConfig, depositConfigStructBob);

    // Check the TokenVaults after deposits
    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query_1 = `{
      tokenVaultInputAlice: tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
      tokenVaultOutputAlice: tokenVault (id: "${tokenVault_Output_Alice_ID}") {
        balance
      }
      tokenVaultInputBob: tokenVault (id: "${tokenVault_Input_Bob_ID}") {
        balance
      }
      tokenVaultOutputBob: tokenVault (id: "${tokenVault_Output_Bob_ID}") {
        balance
      }
    }`;

    const response_1 = (await subgraph({ query: query_1 })) as FetchResult;

    const data_1 = response_1.data;

    // Alice check
    assert.equal(
      data_1.tokenVaultInputAlice.balance,
      0,
      "Wrong: Input TokenVault was updated"
    );
    assert.equal(
      data_1.tokenVaultOutputAlice.balance,
      depositAliceConfig.amount.toString(),
      "Wrong: Output TokenVault was not updated"
    );

    // Bob check
    assert.equal(
      data_1.tokenVaultInputBob.balance,
      0,
      "Wrong: Input TokenVault was updated"
    );
    assert.equal(
      data_1.tokenVaultOutputBob.balance,
      depositBobConfig.amount.toString(),
      "Wrong: Output TokenVault was not updated"
    );
  });

  it("should update the TokenVaults after withdrawal", async function () {
    const [, alice] = signers;

    const vaultId = ethers.BigNumber.from(1);
    // - Token Vault ID:
    const tokenVault_Input_Alice_ID = `${vaultId.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    // Tracker balance vault
    let aliceVaultTracker = ethers.constants.Zero;

    // DEPOSIT
    const amount = ethers.BigNumber.from("1000" + eighteenZeros);
    await tokenA.transfer(alice.address, amount);

    const depositConfigStruct: DepositConfigStruct = {
      token: tokenA.address,
      vaultId,
      amount,
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

    // Updating variable tracker (it's adding since is a deposit)
    aliceVaultTracker = aliceVaultTracker.add(depositConfig.amount);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query_0 = `{
      tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
    }`;

    const response_0 = (await subgraph({ query: query_0 })) as FetchResult;

    const data_0 = response_0.data.tokenVault;

    assert.equal(
      data_0.balance,
      aliceVaultTracker.toString(),
      "Balance not updated after deposit"
    );

    // WITHDRAW
    const aliceTokenABalance0 = await tokenA.balanceOf(alice.address);

    const withdrawConfigStruct: WithdrawConfigStruct = {
      token: tokenA.address,
      vaultId: vaultId,
      amount,
    };

    const txWithdraw = await orderBook
      .connect(alice)
      .withdraw(withdrawConfigStruct);

    const { sender: withdrawSender, config: withdrawConfig } =
      (await getEventArgs(
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

    // Updating variable tracker (it's substrasting since is a withdraw)
    aliceVaultTracker = aliceVaultTracker.sub(withdrawConfig.amount);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query_1 = `{
      tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
    }`;

    const response_1 = (await subgraph({ query: query_1 })) as FetchResult;

    const data_1 = response_1.data.tokenVault;

    assert.equal(
      data_1.balance,
      aliceVaultTracker.toString(),
      "Balance not updated after withdraw"
    );
  });

  it("should update the balance of the TokenVault after clearing orders", async function () {
    const [, alice, bob, bountyBot] = signers;

    // Variables to track the changes on the vaults
    let aliceInputVaultTracker = ethers.constants.Zero;
    let aliceOutputVaultTracker = ethers.constants.Zero;
    let bobInputVaultTracker = ethers.constants.Zero;
    let bobOutputVaultTracker = ethers.constants.Zero;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const bobInputVault = ethers.BigNumber.from(randomUint256());
    const bobOutputVault = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultA = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultB = ethers.BigNumber.from(randomUint256());

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const OrderConfig_A: OrderConfigStruct = await getOrderConfig(
      ratio_A,
      max_uint256,
      tokenA.address,
      18,
      aliceInputVault,
      tokenB.address,
      18,
      aliceOutputVault,
      aliceOrder
    );

    const txAddOrderAlice = await orderBook
      .connect(alice)
      .addOrder(OrderConfig_A);

    const { sender: sender_A, order: Order_A } = (await getEventArgs(
      txAddOrderAlice,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_A === alice.address, "wrong sender");
    compareStructs(Order_A, OrderConfig_A);

    // Order_B
    const ratio_B = fixedPointDiv(ONE, ratio_A);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const bobOrder = encodeMeta("Order_B");

    const OrderConfig_B: OrderConfigStruct = await getOrderConfig(
      ratio_B,
      max_uint256,
      tokenB.address,
      18,
      bobInputVault,
      tokenA.address,
      18,
      bobOutputVault,
      bobOrder
    );

    const txAddOrderBob = await orderBook.connect(bob).addOrder(OrderConfig_B);

    const { sender: sender_B, order: Order_B } = (await getEventArgs(
      txAddOrderBob,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_B === bob.address, "wrong sender");
    compareStructs(Order_B, OrderConfig_B);

    // Checking SG entity after adding order to confirm status
    // Subgraph check
    await waitForSubgraphToBeSynced();

    // Alice Data IDs: (across the test)
    // - Token Vault:
    const tokenVault_Input_Alice_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    const tokenVault_Output_Alice_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;

    // Bob Data IDs:(across the test)
    // - Token Vault:
    const tokenVault_Input_Bob_ID = `${bobInputVault.toString()}-${bob.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;
    const tokenVault_Output_Bob_ID = `${bobOutputVault.toString()}-${bob.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;

    const query_0 = `{
      tokenVaultInputAlice: tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
      tokenVaultOutputAlice: tokenVault (id: "${tokenVault_Output_Alice_ID}") {
        balance
      }
      tokenVaultInputBob: tokenVault (id: "${tokenVault_Input_Bob_ID}") {
        balance
      }
      tokenVaultOutputBob: tokenVault (id: "${tokenVault_Output_Bob_ID}") {
        balance
      }
    }`;

    const response_0 = (await subgraph({ query: query_0 })) as FetchResult;

    const data_0 = response_0.data;

    // Alice check
    assert.equal(
      data_0.tokenVaultInputAlice.balance,
      aliceInputVaultTracker.toString()
    );
    assert.equal(
      data_0.tokenVaultOutputAlice.balance,
      aliceOutputVaultTracker.toString()
    );

    // Bob check
    assert.equal(
      data_0.tokenVaultInputBob.balance,
      bobInputVaultTracker.toString()
    );
    assert.equal(
      data_0.tokenVaultOutputBob.balance,
      bobOutputVaultTracker.toString()
    );

    ////////////
    // DEPOSITS
    const amountB = ethers.BigNumber.from("1000" + eighteenZeros);
    const amountA = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenB.transfer(alice.address, amountB);
    await tokenA.transfer(bob.address, amountA);

    const depositConfigStructAlice: DepositConfigStruct = {
      token: tokenB.address,
      vaultId: aliceOutputVault,
      amount: amountB,
    };
    const depositConfigStructBob: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: bobOutputVault,
      amount: amountA,
    };

    await tokenB
      .connect(alice)
      .approve(orderBook.address, depositConfigStructAlice.amount);
    await tokenA
      .connect(bob)
      .approve(orderBook.address, depositConfigStructBob.amount);

    // Alice deposits tokenB into her output vault
    const txDepositOrderAlice = await orderBook
      .connect(alice)
      .deposit(depositConfigStructAlice);
    // Bob deposits tokenA into his output vault
    const txDepositOrderBob = await orderBook
      .connect(bob)
      .deposit(depositConfigStructBob);

    const { sender: depositAliceSender, config: depositAliceConfig } =
      (await getEventArgs(
        txDepositOrderAlice,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];
    const { sender: depositBobSender, config: depositBobConfig } =
      (await getEventArgs(
        txDepositOrderBob,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    assert(depositAliceSender === alice.address);
    compareStructs(depositAliceConfig, depositConfigStructAlice);
    assert(depositBobSender === bob.address);
    compareStructs(depositBobConfig, depositConfigStructBob);

    // Updating the variables tracker for the balances.
    // ONLY both outputs vaults for alice and bob were updated
    aliceOutputVaultTracker = aliceOutputVaultTracker.add(
      depositAliceConfig.amount
    );
    bobOutputVaultTracker = bobOutputVaultTracker.add(depositBobConfig.amount);

    // Check the TokenVaults after deposits
    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query_1 = `{
      tokenVaultInputAlice: tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
      tokenVaultOutputAlice: tokenVault (id: "${tokenVault_Output_Alice_ID}") {
        balance
      }
      tokenVaultInputBob: tokenVault (id: "${tokenVault_Input_Bob_ID}") {
        balance
      }
      tokenVaultOutputBob: tokenVault (id: "${tokenVault_Output_Bob_ID}") {
        balance
      }
    }`;

    const response_1 = (await subgraph({ query: query_1 })) as FetchResult;

    const data_1 = response_1.data;

    // Alice check
    assert.equal(
      data_1.tokenVaultInputAlice.balance,
      aliceInputVaultTracker.toString(),
      "Wrong: Input TokenVault was updated"
    );
    assert.equal(
      data_1.tokenVaultOutputAlice.balance,
      aliceOutputVaultTracker.toString(),
      "Wrong: Output TokenVault was not updated"
    );

    // Bob check
    assert.equal(
      data_1.tokenVaultInputBob.balance,
      bobInputVaultTracker.toString(),
      "Wrong: Input TokenVault was updated"
    );
    assert.equal(
      data_1.tokenVaultOutputBob.balance,
      bobOutputVaultTracker.toString(),
      "Wrong: Output TokenVault was not updated"
    );

    // BOUNTY BOT CLEARS THE ORDER

    const clearConfig: ClearConfigStruct = {
      aliceInputIOIndex: 0,
      aliceOutputIOIndex: 0,
      bobInputIOIndex: 0,
      bobOutputIOIndex: 0,
      aliceBountyVaultId: bountyBotVaultA,
      bobBountyVaultId: bountyBotVaultB,
    };

    const txClearOrder = await orderBook
      .connect(bountyBot)
      .clear(Order_A, Order_B, clearConfig, [], []);

    const {
      sender: clearSender,
      alice: clearA_,
      bob: clearB_,
      clearConfig: clearBountyConfig,
    } = (await getEventArgs(
      txClearOrder,
      "Clear",
      orderBook
    )) as ClearEvent["args"];

    const { sender: afterClearSender, clearStateChange: clearStateChange } =
      (await getEventArgs(
        txClearOrder,
        "AfterClear",
        orderBook
      )) as AfterClearEvent["args"];

    const aOutputMaxExpected = amountA;
    const bOutputMaxExpected = amountB;

    const aOutputExpected = minBN(
      aOutputMaxExpected,
      fixedPointMul(ratio_B, amountA)
    );
    const bOutputExpected = minBN(
      bOutputMaxExpected,
      fixedPointMul(ratio_A, amountB)
    );

    const expectedClearStateChange: ClearStateChangeStruct = {
      aliceOutput: aOutputExpected,
      bobOutput: bOutputExpected,
      aliceInput: fixedPointMul(ratio_A, aOutputExpected),
      bobInput: fixedPointMul(ratio_B, bOutputExpected),
    };

    assert(afterClearSender === bountyBot.address);
    assert(clearSender === bountyBot.address);
    compareSolStructs(clearA_, Order_A);
    compareSolStructs(clearB_, Order_B);
    compareStructs(clearBountyConfig, clearConfig);
    compareStructs(clearStateChange, expectedClearStateChange);

    // Updating the variables tracker for the balances.
    // The output vaults are subtracted but the input vaults are added
    // - Alice vaults
    aliceInputVaultTracker = aliceInputVaultTracker.add(
      clearStateChange.aliceInput
    );
    aliceOutputVaultTracker = aliceOutputVaultTracker.sub(
      clearStateChange.aliceOutput
    );
    // - Bob vaults
    bobInputVaultTracker = bobInputVaultTracker.add(clearStateChange.bobInput);
    bobOutputVaultTracker = bobOutputVaultTracker.sub(
      clearStateChange.bobOutput
    );

    // Check the final tracker balance against the balance that the OB hold.
    const vaultBalAliceInput = await orderBook.vaultBalance(
      alice.address,
      tokenA.address,
      aliceInputVault
    );
    const vaultBalAliceOutput = await orderBook.vaultBalance(
      alice.address,
      tokenB.address,
      aliceOutputVault
    );

    const vaultBalBobInput = await orderBook.vaultBalance(
      bob.address,
      tokenB.address,
      bobInputVault
    );
    const vaultBalBobOutput = await orderBook.vaultBalance(
      bob.address,
      tokenA.address,
      bobOutputVault
    );

    assert(
      vaultBalAliceInput.eq(aliceInputVaultTracker),
      "Wrong: Vault Balance Alice Input Tracker"
    );
    assert(
      vaultBalAliceOutput.eq(aliceOutputVaultTracker),
      "Wrong: Vault Balance Alice Output Tracker"
    );

    assert(
      vaultBalBobInput.eq(bobInputVaultTracker),
      "Wrong: Vault Balance Bob Input Tracker"
    );
    assert(
      vaultBalBobOutput.eq(bobOutputVaultTracker),
      "Wrong: Vault Balance Bob Output Tracker"
    );

    // Check the TokenVaults after Clearing
    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query_2 = `{
      tokenVaultInputAlice: tokenVault (id: "${tokenVault_Input_Alice_ID}") {
        balance
      }
      tokenVaultOutputAlice: tokenVault (id: "${tokenVault_Output_Alice_ID}") {
        balance
      }
      tokenVaultInputBob: tokenVault (id: "${tokenVault_Input_Bob_ID}") {
        balance
      }
      tokenVaultOutputBob: tokenVault (id: "${tokenVault_Output_Bob_ID}") {
        balance
      }
    }`;

    const response_2 = (await subgraph({ query: query_2 })) as FetchResult;

    const data_2 = response_2.data;

    // Alice check
    assert.equal(
      data_2.tokenVaultInputAlice.balance,
      aliceInputVaultTracker.toString()
    );
    assert.equal(
      data_2.tokenVaultOutputAlice.balance,
      aliceOutputVaultTracker.toString()
    );

    // Bob check
    assert.equal(
      data_2.tokenVaultInputBob.balance,
      bobInputVaultTracker.toString()
    );
    assert.equal(
      data_2.tokenVaultOutputBob.balance,
      bobOutputVaultTracker.toString()
    );
  });

  it("should add the ClearOrder to the TokenVault after clearing orders", async function () {
    const [, alice, bob, bountyBot] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const bobInputVault = ethers.BigNumber.from(randomUint256());
    const bobOutputVault = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultA = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultB = ethers.BigNumber.from(randomUint256());

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const OrderConfig_A: OrderConfigStruct = await getOrderConfig(
      ratio_A,
      max_uint256,
      tokenA.address,
      18,
      aliceInputVault,
      tokenB.address,
      18,
      aliceOutputVault,
      aliceOrder
    );

    const txAddOrderAlice = await orderBook
      .connect(alice)
      .addOrder(OrderConfig_A);

    const { sender: sender_A, order: Order_A } = (await getEventArgs(
      txAddOrderAlice,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_A === alice.address, "wrong sender");
    compareStructs(Order_A, OrderConfig_A);

    // Order_B
    const ratio_B = fixedPointDiv(ONE, ratio_A);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const bobOrder = encodeMeta("Order_B");

    const OrderConfig_B: OrderConfigStruct = await getOrderConfig(
      ratio_B,
      max_uint256,
      tokenB.address,
      18,
      bobInputVault,
      tokenA.address,
      18,
      bobOutputVault,
      bobOrder
    );

    const txAddOrderBob = await orderBook.connect(bob).addOrder(OrderConfig_B);

    const { sender: sender_B, order: Order_B } = (await getEventArgs(
      txAddOrderBob,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_B === bob.address, "wrong sender");
    compareStructs(Order_B, OrderConfig_B);

    // DEPOSITS
    const amountB = ethers.BigNumber.from("1000" + eighteenZeros);
    const amountA = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenB.transfer(alice.address, amountB);
    await tokenA.transfer(bob.address, amountA);

    const depositConfigStructAlice: DepositConfigStruct = {
      token: tokenB.address,
      vaultId: aliceOutputVault,
      amount: amountB,
    };
    const depositConfigStructBob: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: bobOutputVault,
      amount: amountA,
    };

    await tokenB
      .connect(alice)
      .approve(orderBook.address, depositConfigStructAlice.amount);
    await tokenA
      .connect(bob)
      .approve(orderBook.address, depositConfigStructBob.amount);

    // Alice deposits tokenB into her output vault
    const txDepositOrderAlice = await orderBook
      .connect(alice)
      .deposit(depositConfigStructAlice);
    // Bob deposits tokenA into his output vault
    const txDepositOrderBob = await orderBook
      .connect(bob)
      .deposit(depositConfigStructBob);

    const { sender: depositAliceSender, config: depositAliceConfig } =
      (await getEventArgs(
        txDepositOrderAlice,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];
    const { sender: depositBobSender, config: depositBobConfig } =
      (await getEventArgs(
        txDepositOrderBob,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    assert(depositAliceSender === alice.address);
    compareStructs(depositAliceConfig, depositConfigStructAlice);
    assert(depositBobSender === bob.address);
    compareStructs(depositBobConfig, depositConfigStructBob);

    // BOUNTY BOT CLEARS THE ORDER

    const clearConfig: ClearConfigStruct = {
      aliceInputIOIndex: 0,
      aliceOutputIOIndex: 0,
      bobInputIOIndex: 0,
      bobOutputIOIndex: 0,
      aliceBountyVaultId: bountyBotVaultA,
      bobBountyVaultId: bountyBotVaultB,
    };

    const txClearOrder = await orderBook
      .connect(bountyBot)
      .clear(Order_A, Order_B, clearConfig, [], []);

    const {
      sender: clearSender,
      alice: clearA_,
      bob: clearB_,
      clearConfig: clearBountyConfig,
    } = (await getEventArgs(
      txClearOrder,
      "Clear",
      orderBook
    )) as ClearEvent["args"];
    const { sender: afterClearSender, clearStateChange: clearStateChange } =
      (await getEventArgs(
        txClearOrder,
        "AfterClear",
        orderBook
      )) as AfterClearEvent["args"];

    const aOutputMaxExpected = amountA;
    const bOutputMaxExpected = amountB;

    const aOutputExpected = minBN(
      aOutputMaxExpected,
      fixedPointMul(ratio_B, amountA)
    );
    const bOutputExpected = minBN(
      bOutputMaxExpected,
      fixedPointMul(ratio_A, amountB)
    );

    const expectedClearStateChange: ClearStateChangeStruct = {
      aliceOutput: aOutputExpected,
      bobOutput: bOutputExpected,
      aliceInput: fixedPointMul(ratio_A, aOutputExpected),
      bobInput: fixedPointMul(ratio_B, bOutputExpected),
    };

    assert(afterClearSender === bountyBot.address);
    assert(clearSender === bountyBot.address);
    compareSolStructs(clearA_, Order_A);
    compareSolStructs(clearB_, Order_B);
    compareStructs(clearBountyConfig, clearConfig);
    compareStructs(clearStateChange, expectedClearStateChange);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    // TokenVault: #{vaultId}-{owner}-{token}
    const tokenVault_Input_ID = `${aliceInputVault.toString()}-${alice.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;
    const tokenVault_Output_ID = `${aliceOutputVault.toString()}-${alice.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;

    const clearOrder_ID = `${txClearOrder.hash}-0`;

    const query = `{
      tokenVaultInput: tokenVault (id: "${tokenVault_Input_ID}") {
        orderClears {
          id
        }
      }
      tokenVaultOutput: tokenVault (id: "${tokenVault_Output_ID}") {
        orderClears {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const dataInput = response.data.tokenVaultInput;
    const dataOutput = response.data.tokenVaultOutput;

    // TODO: Rework on the OrderClear entity for their ID @vishal @naneez
    expect(dataInput.orderClears).to.deep.include({
      id: clearOrder_ID,
    });
    expect(dataOutput.orderClears).to.deep.include({
      id: clearOrder_ID,
    });
  });

  it("should create/update the TokenVault of the Clearer address", async function () {
    const [, alice, bob, bountyBot] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const bobInputVault = ethers.BigNumber.from(randomUint256());
    const bobOutputVault = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultA = ethers.BigNumber.from(randomUint256());
    const bountyBotVaultB = ethers.BigNumber.from(randomUint256());

    // Order_A
    const ratio_A = ethers.BigNumber.from("90" + eighteenZeros);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const OrderConfig_A: OrderConfigStruct = await getOrderConfig(
      ratio_A,
      max_uint256,
      tokenA.address,
      18,
      aliceInputVault,
      tokenB.address,
      18,
      aliceOutputVault,
      aliceOrder
    );

    const txAddOrderAlice = await orderBook
      .connect(alice)
      .addOrder(OrderConfig_A);

    const { sender: sender_A, order: Order_A } = (await getEventArgs(
      txAddOrderAlice,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_A === alice.address, "wrong sender");
    compareStructs(Order_A, OrderConfig_A);

    // Order_B
    const ratio_B = fixedPointDiv(ONE, ratio_A);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const bobOrder = encodeMeta("Order_B");

    const OrderConfig_B: OrderConfigStruct = await getOrderConfig(
      ratio_B,
      max_uint256,
      tokenB.address,
      18,
      bobInputVault,
      tokenA.address,
      18,
      bobOutputVault,
      bobOrder
    );

    const txAddOrderBob = await orderBook.connect(bob).addOrder(OrderConfig_B);

    const { sender: sender_B, order: Order_B } = (await getEventArgs(
      txAddOrderBob,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_B === bob.address, "wrong sender");
    compareStructs(Order_B, OrderConfig_B);

    // DEPOSITS
    const amountB = ethers.BigNumber.from("1000" + eighteenZeros);
    const amountA = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenB.transfer(alice.address, amountB);
    await tokenA.transfer(bob.address, amountA);

    const depositConfigStructAlice: DepositConfigStruct = {
      token: tokenB.address,
      vaultId: aliceOutputVault,
      amount: amountB,
    };
    const depositConfigStructBob: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: bobOutputVault,
      amount: amountA,
    };

    await tokenB
      .connect(alice)
      .approve(orderBook.address, depositConfigStructAlice.amount);
    await tokenA
      .connect(bob)
      .approve(orderBook.address, depositConfigStructBob.amount);

    // Alice deposits tokenB into her output vault
    const txDepositOrderAlice = await orderBook
      .connect(alice)
      .deposit(depositConfigStructAlice);
    // Bob deposits tokenA into his output vault
    const txDepositOrderBob = await orderBook
      .connect(bob)
      .deposit(depositConfigStructBob);

    const { sender: depositAliceSender, config: depositAliceConfig } =
      (await getEventArgs(
        txDepositOrderAlice,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];
    const { sender: depositBobSender, config: depositBobConfig } =
      (await getEventArgs(
        txDepositOrderBob,
        "Deposit",
        orderBook
      )) as DepositEvent["args"];

    assert(depositAliceSender === alice.address);
    compareStructs(depositAliceConfig, depositConfigStructAlice);
    assert(depositBobSender === bob.address);
    compareStructs(depositBobConfig, depositConfigStructBob);

    // BOUNTY BOT CLEARS THE ORDER

    const clearConfig: ClearConfigStruct = {
      aliceInputIOIndex: 0,
      aliceOutputIOIndex: 0,
      bobInputIOIndex: 0,
      bobOutputIOIndex: 0,
      aliceBountyVaultId: bountyBotVaultA,
      bobBountyVaultId: bountyBotVaultB,
    };

    const txClearOrder = await orderBook
      .connect(bountyBot)
      .clear(Order_A, Order_B, clearConfig, [], []);

    const {
      sender: clearSender,
      alice: clearA_,
      bob: clearB_,
      clearConfig: clearBountyConfig,
    } = (await getEventArgs(
      txClearOrder,
      "Clear",
      orderBook
    )) as ClearEvent["args"];
    const { sender: afterClearSender, clearStateChange: clearStateChange } =
      (await getEventArgs(
        txClearOrder,
        "AfterClear",
        orderBook
      )) as AfterClearEvent["args"];

    const aOutputMaxExpected = amountA;
    const bOutputMaxExpected = amountB;

    const aOutputExpected = minBN(
      aOutputMaxExpected,
      fixedPointMul(ratio_B, amountA)
    );
    const bOutputExpected = minBN(
      bOutputMaxExpected,
      fixedPointMul(ratio_A, amountB)
    );

    const expectedClearStateChange: ClearStateChangeStruct = {
      aliceOutput: aOutputExpected,
      bobOutput: bOutputExpected,
      aliceInput: fixedPointMul(ratio_A, aOutputExpected),
      bobInput: fixedPointMul(ratio_B, bOutputExpected),
    };

    assert(afterClearSender === bountyBot.address);
    assert(clearSender === bountyBot.address);
    compareSolStructs(clearA_, Order_A);
    compareSolStructs(clearB_, Order_B);
    compareStructs(clearBountyConfig, clearConfig);
    compareStructs(clearStateChange, expectedClearStateChange);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    // Vault ID where the bounty will be move
    const { aliceBountyVaultId, bobBountyVaultId } = clearBountyConfig;

    // TokenVault: #{vaultId}-{owner}-{token}
    const tokenVault_A_ID = `${aliceBountyVaultId.toString()}-${bountyBot.address.toLowerCase()}-${tokenB.address.toLowerCase()}`;
    const tokenVault_B_ID = `${bobBountyVaultId.toString()}-${bountyBot.address.toLowerCase()}-${tokenA.address.toLowerCase()}`;

    const vault_A_ID = `${aliceBountyVaultId.toString()}-${bountyBot.address.toLowerCase()}`;
    const vault_B_ID = `${bobBountyVaultId.toString()}-${bountyBot.address.toLowerCase()}`;

    const vaultBalance_A = await orderBook.vaultBalance(
      bountyBot.address,
      tokenB.address,
      aliceBountyVaultId
    );
    const vaultBalance_B = await orderBook.vaultBalance(
      bountyBot.address,
      tokenA.address,
      bobBountyVaultId
    );

    const query = `{
      tokenVault_A: tokenVault (id: "${tokenVault_A_ID}") {
        balance
        owner {
          id
        }
        vault {
          id
        }
        token {
          id
        }
      }
      tokenVault_B: tokenVault (id: "${tokenVault_B_ID}") {
        balance
        owner {
          id
        }
        vault {
          id
        }
        token {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data_A = response.data.tokenVault_A;
    const data_B = response.data.tokenVault_B;

    // TokenVault A
    assert.equal(data_A.balance, vaultBalance_A.toString());
    assert.equal(data_A.owner.id, bountyBot.address.toLowerCase());
    assert.equal(data_A.vault.id, vault_A_ID);
    assert.equal(data_A.token.id, tokenB.address.toLowerCase());

    // TokenVault B
    assert.equal(data_B.balance, vaultBalance_B.toString());
    assert.equal(data_B.owner.id, bountyBot.address.toLowerCase());
    assert.equal(data_B.vault.id, vault_B_ID);
    assert.equal(data_B.token.id, tokenA.address.toLowerCase());
  });
});
