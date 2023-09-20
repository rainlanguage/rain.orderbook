import assert from "assert";
import { expect } from "chai";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { ReserveToken18 } from "../typechain";
import {
  ONE,
  basicDeploy,
  compareSolStructs,
  compareStructs,
  eighteenZeros,
  fixedPointDiv,
  fixedPointMul,
  getEvents,
  max_uint256,
  minBN,
  randomUint256,
} from "../utils";
import { ethers } from "hardhat";
import { encodeMeta, getOrderConfig } from "../utils/orderBook/order";
import {
  AddOrderEvent,
  AfterClearEvent,
  ClearConfigStruct,
  ClearEvent,
  ClearStateChangeStruct,
  DepositConfigStruct,
  DepositEvent,
  OrderConfigStruct,
} from "../typechain/contracts/orderbook/OrderBook";
import { MultiClearStruct } from "../typechain/contracts/test/orderbook/OBMultiTx";
import {
  deployOBMultiTx,
  getEventArgs,
  waitForSubgraphToBeSynced,
} from "./utils";

describe("OrderClear entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    await tokenA.initialize();
    await tokenB.initialize();
  });

  it("should query ClearOrder after clearing orders", async function () {
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

    const {
      sender: sender_A,
      order: Order_A,
      orderHash: orderHash_A,
    } = (await getEventArgs(
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

    const {
      sender: sender_B,
      order: Order_B,
      orderHash: orderHash_B,
    } = (await getEventArgs(
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

    const orderClear_ID = `${txClearOrder.hash.toLowerCase()}-0`;

    const query = `{
      orderClear (id: "${orderClear_ID}") {
        aInputIOIndex
        aOutputIOIndex
        bInputIOIndex
        bOutputIOIndex
        sender {
          id
        }
        clearer {
          id
        }
        bounty {
          id
        }
        stateChange {
          id
        }
        orderA {
          id
        }
        orderB {
          id
        }
        owners {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;
    const data = response.data.orderClear;

    // Index of Input/Output values
    assert.equal(data.aInputIOIndex, clearBountyConfig.aliceInputIOIndex);
    assert.equal(data.aOutputIOIndex, clearBountyConfig.aliceOutputIOIndex);
    assert.equal(data.bInputIOIndex, clearBountyConfig.bobInputIOIndex);
    assert.equal(data.bOutputIOIndex, clearBountyConfig.bobOutputIOIndex);

    assert.equal(data.sender.id, bountyBot.address.toLowerCase());
    assert.equal(data.clearer.id, bountyBot.address.toLowerCase());

    // The ID is generated by the SG. Only need to exist
    expect(data.bounty.id).to.be.not.null;
    expect(data.stateChange.id).to.be.not.null;

    assert.equal(data.orderA.id, orderHash_A.toHexString());
    assert.equal(data.orderB.id, orderHash_B.toHexString());

    expect(data.owners).to.deep.include({
      id: clearA_.owner.toLowerCase(),
    });
    expect(data.owners).to.deep.include({
      id: clearB_.owner.toLowerCase(),
    });
  });

  it("should query correctly ClearOrder after clearing orders on same transactions", async function () {
    const [, alice, bob] = signers;

    // Deploy new OBMultiTx contract
    const obMultiTx = await deployOBMultiTx(orderBook);

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

    // OBMultiTx contract CLEARS THE ORDER
    const clearConfig: ClearConfigStruct = {
      aliceInputIOIndex: 0,
      aliceOutputIOIndex: 0,
      bobInputIOIndex: 0,
      bobOutputIOIndex: 0,
      aliceBountyVaultId: bountyBotVaultA,
      bobBountyVaultId: bountyBotVaultB,
    };

    // Generate the Config to sent the multiClear
    const config: MultiClearStruct = {
      alice_: Order_A,
      bob_: Order_B,
      clearConfig_: clearConfig,
      aliceSignedContext_: [],
      bobSignedContext_: [],
    };

    // Use the same config two times
    const configMultiClear: MultiClearStruct[] = [config, config];

    const txMultiClear = await obMultiTx
      .connect(alice)
      .multiClear(configMultiClear);

    const clearEvents = (await getEvents(
      txMultiClear,
      "Clear",
      orderBook
    )) as Array<ClearEvent["args"]>;

    const afterClearEvents = (await getEvents(
      txMultiClear,
      "AfterClear",
      orderBook
    )) as Array<AfterClearEvent["args"]>;

    assert(clearEvents.length === afterClearEvents.length);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    for (let i = 0; i < clearEvents.length; i++) {
      const orderClear_ID = `${txMultiClear.hash.toLowerCase()}-${i}`;
      const { clearConfig } = clearEvents[i];

      const query = `{
        orderClear (id: "${orderClear_ID}") {
          aInputIOIndex
          aOutputIOIndex
          bInputIOIndex
          bOutputIOIndex
        }
      }`;
      const response = (await subgraph({ query })) as FetchResult;
      const data = response.data.orderClear;

      // Index of Input/Output values
      assert.equal(data.aInputIOIndex, clearConfig.aliceInputIOIndex);
      assert.equal(data.aOutputIOIndex, clearConfig.aliceOutputIOIndex);
      assert.equal(data.bInputIOIndex, clearConfig.bobInputIOIndex);
      assert.equal(data.bOutputIOIndex, clearConfig.bobOutputIOIndex);
    }
  });
});
