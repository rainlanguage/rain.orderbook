import assert from "assert";
import { expect } from "chai";
import { FetchResult } from "apollo-fetch";
import { orderBook, signers, subgraph } from "./0_initialization.test";
import { RainterpreterExpressionDeployer, ReserveToken18 } from "../typechain";
import {
  MemoryType,
  ONE,
  Opcode,
  basicDeploy,
  compareStructs,
  eighteenZeros,
  fixedPointDiv,
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
  IOStructOutput,
  OrderConfigStruct,
  RemoveOrderEvent,
} from "../typechain/contracts/orderbook/OrderBook";
import {
  getEventArgs,
  getTxTimeblock,
  waitForSubgraphToBeSynced,
} from "./utils";

async function getInterpretersFromDeployer(deployerAddress: string) {
  const expressionDeployer = (await ethers.getContractAt(
    "RainterpreterExpressionDeployer",
    deployerAddress
  )) as RainterpreterExpressionDeployer;

  return {
    deployer: deployerAddress,
    store: await expressionDeployer.store(),
    rainterpreter: await expressionDeployer.interpreter(),
  };
}

/**
 * @param hexOrderHash_ Order hash emitted
 * @param arrayIO_ The order IO data (input OR output)
 * @param dataIO_ The arrays of (input OR output) ID's to check agaisnt `arrayIO_`
 */
function checkIO(
  hexOrderHash_: string,
  arrayIO_: IOStructOutput[],
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  dataIO_: any
) {
  const length = arrayIO_.length;
  for (let i = 0; i < length; i++) {
    const IOToken = arrayIO_[i].token.toLowerCase();
    const IOVault = arrayIO_[i].vaultId;

    // ID = `order.hash() - IO.token - IO.vaultId`
    const IO_ID = `${hexOrderHash_.toLowerCase()}-${IOToken}-${IOVault}`;
    expect(dataIO_).to.deep.include({
      id: IO_ID,
    });
  }
}

describe("Order entity", () => {
  let tokenA: ReserveToken18;
  let tokenB: ReserveToken18;

  beforeEach(async () => {
    tokenA = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
    tokenB = (await basicDeploy("ReserveToken18", {})) as ReserveToken18;
  });

  it("should query the Order after adding an order", async () => {
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

    await waitForSubgraphToBeSynced();

    // Subgraph check
    // Get Interpreters from ExpressionDeployer address

    const { deployer, store, rainterpreter } =
      await getInterpretersFromDeployer(ExpressionDeployer_A);

    const [, addTimestamp] = await getTxTimeblock(txOrder_A);

    const query = `{
      order (id: "${orderHash.toHexString().toLowerCase()}") {
        transaction{
          id
        }
        owner{
          id
        }
        interpreter
        interpreterStore
        expressionDeployer
        expression
        orderActive
        handleIO
        timestamp
        meta {
          id
        }
        validInputs {
          id
        }
        validOutputs {
          id
        }
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data = response.data.order;

    assert.equal(
      data.transaction.id.toLowerCase(),
      txOrder_A.hash.toLowerCase()
    );
    assert.equal(data.owner.id, sender_A.toLowerCase());

    assert.equal(data.interpreter, rainterpreter.toLowerCase());
    assert.equal(data.interpreterStore, store.toLowerCase());
    assert.equal(data.expressionDeployer, deployer.toLowerCase());
    assert.equal(
      data.expression,
      order_A.evaluable.expression.toLowerCase(),
      "Wrong expression address"
    );
    assert.equal(data.orderActive, true);
    assert.equal(data.handleIO, order_A.handleIO);
    assert.equal(data.timestamp, addTimestamp);

    // TODO: Add suport for MetaV1 entities @naneez
    // assert.equal(data.meta.id, '');

    // Checking every validInputs
    checkIO(orderHash.toHexString(), order_A.validInputs, data.validInputs);

    // Checking every validOutputs
    checkIO(orderHash.toHexString(), order_A.validOutputs, data.validOutputs);
  });

  it("should query multiple Orders when adding orders", async () => {
    const [, alice, bob] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());
    const bobInputVault = ethers.BigNumber.from(randomUint256());
    const bobOutputVault = ethers.BigNumber.from(randomUint256());

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
      orderHash: orderHash_A,
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

    // Order_B

    const ratio_B = fixedPointDiv(ONE, ratio_A);
    const constants_B = [max_uint256, ratio_B];
    const bOpMax = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 0)
    );
    const bRatio = op(
      Opcode.read_memory,
      memoryOperand(MemoryType.Constant, 1)
    );
    // prettier-ignore
    const source_B = concat([
      bOpMax,
      bRatio,
    ]);

    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const bobOrder = encodeMeta("Order_B");

    const EvaluableConfig_B = await generateEvaluableConfig(
      [source_B, []],
      constants_B
    );

    const orderConfig_B: OrderConfigStruct = {
      validInputs: [
        { token: tokenB.address, decimals: 18, vaultId: bobInputVault },
      ],
      validOutputs: [
        { token: tokenA.address, decimals: 18, vaultId: bobOutputVault },
      ],
      evaluableConfig: EvaluableConfig_B,
      meta: bobOrder,
    };

    const txOrderB = await orderBook.connect(bob).addOrder(orderConfig_B);

    const {
      sender: sender_B,
      order: order_B,
      orderHash: orderHash_B,
    } = (await getEventArgs(
      txOrderB,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(sender_B === bob.address, "wrong sender");
    compareStructs(order_B, orderConfig_B);

    await waitForSubgraphToBeSynced();
    // SG check
    const query = `{
      orders {
        id
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data = response.data.orders;

    expect(data).to.deep.include({
      id: orderHash_A.toHexString().toLowerCase(),
    });

    expect(data).to.deep.include({
      id: orderHash_B.toHexString().toLowerCase(),
    });
  });

  it("should update the orderActive field when removing an order", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

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
    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const EvaluableConfig_A = await generateEvaluableConfig(
      [source_A, []],
      constants_A
    );

    const OrderConfig_A: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(OrderConfig_A);

    const {
      sender: liveSender_A,
      order: LiveOrder_A,
      orderHash: addOrderHash,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(liveSender_A === alice.address, "wrong sender");
    compareStructs(LiveOrder_A, OrderConfig_A);

    // REMOVE Order_A

    const txRemoveOrder = await orderBook
      .connect(alice)
      .removeOrder(LiveOrder_A);

    const {
      sender: deadSender_A,
      order: DeadOrder_A,
      orderHash: removeOrderHash,
    } = (await getEventArgs(
      txRemoveOrder,
      "RemoveOrder",
      orderBook
    )) as RemoveOrderEvent["args"];

    await waitForSubgraphToBeSynced();
    assert(deadSender_A === alice.address, "wrong sender");
    compareStructs(DeadOrder_A, OrderConfig_A);

    // SG checks

    assert(addOrderHash.eq(removeOrderHash), "wrong order removed");

    const query = `{
      order (id: "${addOrderHash.toHexString().toLowerCase()}") {
        orderActive
      }
    }`;

    const response = (await subgraph({ query })) as FetchResult;

    const data = response.data.order;

    assert.equal(data.orderActive, false);
  });

  it("should be able to use orderJSONString field to reference the Order directly (remove order)", async () => {
    const [, alice] = signers;

    const aliceInputVault = ethers.BigNumber.from(randomUint256());
    const aliceOutputVault = ethers.BigNumber.from(randomUint256());

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
    // TODO: This is a WRONG encoding meta (FIX: @naneez)
    const aliceOrder = encodeMeta("Order_A");

    const EvaluableConfig_A = await generateEvaluableConfig(
      [source_A, []],
      constants_A
    );

    const OrderConfig_A: OrderConfigStruct = {
      validInputs: [
        { token: tokenA.address, decimals: 18, vaultId: aliceInputVault },
      ],
      validOutputs: [
        { token: tokenB.address, decimals: 18, vaultId: aliceOutputVault },
      ],
      evaluableConfig: EvaluableConfig_A,
      meta: aliceOrder,
    };

    const txOrder_A = await orderBook.connect(alice).addOrder(OrderConfig_A);

    const {
      sender: liveSender_A,
      order: LiveOrder_A,
      orderHash: addOrderHash,
    } = (await getEventArgs(
      txOrder_A,
      "AddOrder",
      orderBook
    )) as AddOrderEvent["args"];

    assert(liveSender_A === alice.address, "wrong sender");
    compareStructs(LiveOrder_A, OrderConfig_A);

    // Wait for sg
    await waitForSubgraphToBeSynced();

    const query_0 = `{
      order (id: "${addOrderHash.toHexString().toLowerCase()}") {
        orderActive
        orderJSONString
      }
    }`;

    const response_0 = (await subgraph({ query: query_0 })) as FetchResult;

    const data_0 = response_0.data.order;

    assert.equal(data_0.orderActive, true);

    const orderFromSg = JSON.parse(data_0.orderJSONString);

    // REMOVE Order_A

    const txRemoveOrder = await orderBook
      .connect(alice)
      .removeOrder(orderFromSg);

    const {
      sender: deadSender_A,
      order: DeadOrder_A,
      orderHash: removeOrderHash,
    } = (await getEventArgs(
      txRemoveOrder,
      "RemoveOrder",
      orderBook
    )) as RemoveOrderEvent["args"];

    await waitForSubgraphToBeSynced();
    assert(deadSender_A === alice.address, "wrong sender");
    compareStructs(DeadOrder_A, OrderConfig_A);

    // SG checks

    assert(addOrderHash.eq(removeOrderHash), "wrong order removed");

    const query_1 = `{
      order (id: "${addOrderHash.toHexString().toLowerCase()}") {
        orderActive
      }
    }`;

    const response_1 = (await subgraph({ query: query_1 })) as FetchResult;

    const data_1 = response_1.data.order;

    assert.equal(data_1.orderActive, false);
  });
});
