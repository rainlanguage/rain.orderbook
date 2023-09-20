import assert from "assert";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { ReserveToken18 } from "../typechain";
import {
  MemoryType,
  Opcode,
  basicDeploy,
  compareStructs,
  eighteenZeros,
  generateEvaluableConfig,
  max_uint256,
  memoryOperand,
  op,
  randomUint256,
} from "../utils";
import { ethers } from "hardhat";
import { encodeMeta } from "../utils/orderBook/order";
import { concat } from "ethers/lib/utils";
import {
  AddOrderEvent,
  DepositConfigStruct,
  DepositEvent,
  OrderConfigStruct,
} from "../typechain/contracts/orderbook/OrderBook";
import { getEventArgs, waitForSubgraphToBeSynced } from "./utils";

describe("ERC20 entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    await tokenA.initialize();
    await tokenB.initialize();
  });

  it("should query the ERC20 after adding an order", async () => {
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

    const query = `{
      tokenA: erc20 (id: "${tokenA.address.toLowerCase()}") {
        name
        symbol
        totalSupply
        decimals
      }
      tokenB: erc20 (id: "${tokenB.address.toLowerCase()}") {
        name
        symbol
        totalSupply
        decimals
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const dataA = response.data.tokenA;
    const dataB = response.data.tokenB;

    // TokenA
    assert.equal(dataA.name, await tokenA.name());
    assert.equal(dataA.symbol, await tokenA.symbol());
    assert.equal(dataA.symbol, await tokenA.symbol());
    assert.equal(dataA.totalSupply, await tokenA.totalSupply());
    assert.equal(dataA.decimals, await tokenA.decimals());

    // TokenB
    assert.equal(dataB.name, await tokenB.name());
    assert.equal(dataB.symbol, await tokenB.symbol());
    assert.equal(dataB.symbol, await tokenB.symbol());
    assert.equal(dataB.totalSupply, await tokenB.totalSupply());
    assert.equal(dataB.decimals, await tokenB.decimals());
  });

  it("should update the Vault after deposits", async function () {
    const [, alice] = signers;

    const vault = ethers.BigNumber.from(randomUint256());
    const amount = ethers.BigNumber.from("1000" + eighteenZeros);

    await tokenA.transfer(alice.address, amount);

    // Deposit config using different tokens
    const depositConfigStruct: DepositConfigStruct = {
      token: tokenA.address,
      vaultId: vault,
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

    // Checking Config
    assert(depositSender === alice.address);
    compareStructs(depositConfig, depositConfigStruct);

    // Subgraph check
    await waitForSubgraphToBeSynced();

    const query = `{
      erc20 (id: "${tokenA.address.toLowerCase()}") {
        name
        symbol
        totalSupply
        decimals
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data = response.data.erc20;

    // Token ERC20
    assert.equal(data.name, await tokenA.name());
    assert.equal(data.symbol, await tokenA.symbol());
    assert.equal(data.symbol, await tokenA.symbol());
    assert.equal(data.totalSupply, await tokenA.totalSupply());
    assert.equal(data.decimals, await tokenA.decimals());
  });

  it("should not break the sg when using a non-ERC20 contract as address when adding an order", async () => {
    const [, alice, , nonErc20_A, nonErc20_B] = signers;

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
        { token: nonErc20_A.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: nonErc20_B.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(orderConfig_A);

    const {
      sender: sender_A,
      expressionDeployer: ExpressionDeployer_A,
      order: order_A,
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

    const query = `{
      nonErc20_A: erc20 (id: "${nonErc20_A.address.toLowerCase()}") {
        name
        symbol
        totalSupply
        decimals
      }
      nonErc20_B: erc20 (id: "${nonErc20_B.address.toLowerCase()}") {
        name
        symbol
        totalSupply
        decimals
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;
    const dataA = response.data.nonErc20_A;
    const dataB = response.data.nonErc20_B;

    // nonErc20_A
    assert.equal(dataA.name, "NONE");
    assert.equal(dataA.symbol, "NONE");
    assert.equal(dataA.totalSupply, 0);
    assert.equal(dataA.decimals, 0);

    // nonErc20_B
    assert.equal(dataB.name, "NONE");
    assert.equal(dataB.symbol, "NONE");
    assert.equal(dataB.totalSupply, 0);
    assert.equal(dataB.decimals, 0);
  });
});
